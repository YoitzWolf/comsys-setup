use argon2::password_hash::SaltString;
use chrono::{DateTime, NaiveDateTime};
use diesel::{ExpressionMethods, Insertable, Queryable, QueryDsl, SelectableHelper};
use diesel::result::Error;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use rand::distributions::{Alphanumeric, DistString};
use crate::r#gen::auth::AuthRequest;
use crate::r#gen::comp::{password_package, JudgeScheme, PasswordPackage};
use crate::gen::comp::{CompDeclaration, CompStatus};
use crate::models::{CompData, CompStaffLink, Competition};
use crate::Permissions;
use argon2::{Argon2, PasswordHasher};
use super::org_mng::{create_com_staff, setup_org_perms_to_user};
use super::time_convert::into_naive;
use rand_core::OsRng;
use super::user_mng::{insert_users, UserModel};

#[derive(Insertable, Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::competitions)]
pub struct CompetitionPrototype {
    pub title: String,
    pub public: bool,
    pub organisation: i32,
    pub start_date: Option<NaiveDateTime>,
    pub ends_date: Option<NaiveDateTime>,
    pub place: Option<String>,
    pub descr: Option<String>,
    pub scheme: i32,
    pub queues: i32,
    pub status: i32,
}

impl From<&CompDeclaration> for CompetitionPrototype {
    fn from(value: &CompDeclaration) -> Self {

        let mut start_date :Option<NaiveDateTime> = None;
        let mut ends_date :Option<NaiveDateTime>  = None;
        if let Some(dates) = value.dates.clone() {
            start_date = into_naive(dates.begins);
            ends_date  = into_naive(dates.ends);
        } 

        Self {
            title: value.title.clone(),
            public: value.public,
            organisation: value.related_organisation_id,
            start_date,
            ends_date,
            place: value.place.clone(),
            descr: value.descr.clone(),
            scheme: value.scheme,
            queues: value.queues.len() as i32,
            status: CompStatus::Declaration as i32,
        }
    }
}

pub async fn insert_competition(conn: &mut AsyncPgConnection, comp: CompetitionPrototype) -> Result<Competition, Error> {
    use crate::schema::competitions::dsl::*;
    diesel::insert_into(
        competitions
    ).values(
        &comp
    ).get_result(conn).await
}

pub async fn insert_comp_data(
    conn: &mut AsyncPgConnection,
    cid: i32,
    queues: Vec<u8>,
    //parts: Vec<u8>
) -> Result<usize, Error> {
    use crate::schema::comp_data::dsl;
    diesel::insert_into(
        dsl::comp_data
    ).values(
        CompData {
            id: cid,
            queues,
            //participants: parts,
        }
    ).execute(conn).await
}

pub async fn get_competition(conn: &mut AsyncPgConnection, cid: i32) -> Result<Competition, Error> {
    use crate::schema::competitions::dsl;
    dsl::competitions
        .filter(dsl::id.eq(cid))
        .select(
            Competition::as_select()
        ).first(conn).await
}

pub async fn get_competitions(conn: &mut AsyncPgConnection, cids: Vec<i32>) -> Result<Vec<Competition>, Error> {
    use crate::schema::competitions::dsl;
    dsl::competitions
        .filter(dsl::id.eq_any(cids))
        .select(
            Competition::as_select()
        ).load(conn).await
}

pub async fn get_public_competitions(conn: &mut AsyncPgConnection, cids: Vec<i32>) -> Result<Vec<Competition>, Error> {
    use crate::schema::competitions::dsl;
    dsl::competitions
        .filter(dsl::id.eq_any(cids))
        .filter(dsl::public.eq(true))
        .select(
            Competition::as_select()
        ).load(conn).await
}


pub async fn get_org_competitions(conn: &mut AsyncPgConnection, orgs: Vec<i32>) -> Result<Vec<Competition>, Error> {
    use crate::schema::competitions::dsl;
    dsl::competitions
        .filter(dsl::organisation.eq_any(orgs))
        .select(
            Competition::as_select()
        ).load(conn).await
}

pub async fn get_public_competitions_all(conn: &mut AsyncPgConnection) -> Result<Vec<Competition>, Error> {
    use crate::schema::competitions::dsl;
    dsl::competitions
        .filter(dsl::public.eq(true))
        .select(
            Competition::as_select()
        ).load(conn).await
}

pub async fn get_comp_data(conn: &mut AsyncPgConnection, cid: i32) -> Result<CompData, Error> {
    use crate::schema::comp_data::dsl;
    dsl::comp_data
        .filter(dsl::id.eq(cid))
        .select(
            CompData::as_select()
        ).first(conn).await
}

pub async fn safe_delete_competition(conn: &mut AsyncPgConnection, cid: i32) -> Result<usize, Error> {
    use crate::schema::competitions::dsl;
    diesel::delete(dsl::competitions).filter(dsl::id.eq(cid)).execute(conn).await
}


pub async fn set_comp_status(conn: &mut AsyncPgConnection, cid: i32, status: CompStatus) -> Result<usize, Error> {
    use crate::schema::competitions::dsl;
    diesel::update(dsl::competitions.find(cid)).set(dsl::status.eq(status as i32)).execute(conn).await
}


pub async fn generate_comp_staff(conn: &mut AsyncPgConnection, cid: i32, declaration: &CompDeclaration) -> Result<PasswordPackage, diesel::result::Error>{
    conn.transaction(
        move |conn| {
            Box::pin(
                async move  {

                    let mut package = PasswordPackage{
                        scheme: declaration.scheme,
                        passwords: vec![]
                    };

                    let result = match JudgeScheme::try_from(declaration.scheme) {
                        Ok(JudgeScheme::SixSixTwo) => {
                            vec![6, 6, 2]
                        }
                        Ok(JudgeScheme::FourFourTwo) => {
                            vec![4, 4, 2]
                        },
                        Ok(JudgeScheme::FourFourOne) => {
                            vec![4, 4, 1]
                        },
                        Err(_) => { panic!("Error scheme checking in comp staff generation process!") },
                    };
                    //println!("> Scheme: {:?}", result);
                    let argon2 = Argon2::default();
                    let mut j_users = vec![];
                    let mut j_perms = vec![];

                    let mut sup_perms = vec![];
                    for qid in 0..declaration.queues.len() {
                        let mut jn_users_markgroup = vec![];
                        for (i, &qsize) in result.iter().enumerate() {
                            let mut jn_users = vec![];
                            for u_num in 0..qsize {
                                let name = format!(
                                        "o{}c{}q{}judge{}u{}{}",
                                        declaration.related_organisation_id,
                                        cid,
                                        qid,
                                        i,
                                        u_num,
                                        Alphanumeric.sample_string(&mut rand::thread_rng(), 5)
                                    );
                                let pwd: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 15).to_ascii_uppercase();
                                jn_users.push(
                                    //UserModel {
                                    //    login: name,
                                    //    hash: argon2.hash_password(pwd.as_bytes(), &SaltString::generate(&mut OsRng)).unwrap().to_string(),
                                    //}
                                    AuthRequest {
                                        login: name,
                                        password: pwd,
                                    }
                                );
                                j_perms.push(Permissions::Judge(cid, qid as i32, i.try_into().unwrap()));
                            }
                            package.passwords.push(password_package::Pack { mark: format!("Judges-queue{qid}-group-{i}").to_owned(), logins: jn_users.clone() });
                            jn_users_markgroup.extend(jn_users);
                        }
                        //package.passwords.push(password_package::Pack { mark: format!("judges-queue{qid}").to_owned(), logins: jn_users.clone() });
                        j_users.extend(jn_users_markgroup);

                        let supervisor = AuthRequest {
                            login: format!(
                                "o{}c{}q{}sup{}",
                                declaration.related_organisation_id,
                                cid,
                                qid,
                                Alphanumeric.sample_string(&mut rand::thread_rng(), 5)
                            ),
                            password: Alphanumeric.sample_string(&mut rand::thread_rng(), 15).to_ascii_uppercase() //hash: argon2.hash_password(Alphanumeric.sample_string(&mut rand::thread_rng(), 20).as_bytes(), &SaltString::generate(&mut OsRng)).unwrap().to_string(),
                        };
                        j_users.push(supervisor.clone());
                        package.passwords.push(password_package::Pack { mark: format!("Arbitor-queue-{qid}").to_owned(), logins: vec![supervisor.clone()] });
                        j_perms.push(Permissions::Arbitor(cid, qid.try_into().unwrap()));
                    }

                    let secretary = AuthRequest {
                        login: format!(
                            "o{}c{}sec{}",
                            declaration.related_organisation_id,
                            cid,
                            Alphanumeric.sample_string(&mut rand::thread_rng(), 7)
                        ),
                        password: Alphanumeric.sample_string(&mut rand::thread_rng(), 15).to_ascii_uppercase() //hash: argon2.hash_password(Alphanumeric.sample_string(&mut rand::thread_rng(), 20).as_bytes(), &SaltString::generate(&mut OsRng)).unwrap().to_string(),
                    };
                    package.passwords.push(password_package::Pack { mark: "secretary".to_owned(), logins: vec![secretary.clone()] });
                    sup_perms.push(Permissions::Secretary(cid));

                    //println!("> Before J. insert");
                    let inserted_judges = insert_users(conn, &j_users.iter().map(|x| { UserModel{
                            selfname: x.login.clone(),
                            login: x.login.clone(),
                            hash: argon2.hash_password(x.password.as_bytes(), &SaltString::generate(&mut OsRng)).unwrap().to_string()
                        } }).collect()
                    ).await?;
                    create_com_staff(conn, &inserted_judges.iter().map(
                        |u| {
                            CompStaffLink {
                                uid: u.id,
                                cid: cid,
                            }
                        }
                    ).collect()).await?;
                    // TODO: Optimize this pls!
                    for (user, perm_qualifier) in inserted_judges.iter().zip(j_perms) {
                        setup_org_perms_to_user(
                            conn,
                            user.id,
                            declaration.related_organisation_id,
                            &vec![
                                perm_qualifier
                            ],
                        ).await?;
                    }
                    //println!("> Perms done!");
                    let inserted_sups = insert_users(conn, &vec![secretary].iter().map(|x| { UserModel{
                            selfname: x.login.clone(),
                            login: x.login.clone(),
                            hash: argon2.hash_password(x.password.as_bytes(), &SaltString::generate(&mut OsRng)).unwrap().to_string()
                        }
                    }).collect() ).await?;

                    for i in inserted_sups.iter().zip(sup_perms) {
                        setup_org_perms_to_user(
                            conn,
                            i.0.id,
                            declaration.related_organisation_id,
                            &vec![
                                i.1
                            ],
                        ).await?;
                    }

                    create_com_staff(conn, &inserted_sups.iter().map(
                        |u| {
                            CompStaffLink {
                                uid: u.id,
                                cid: cid,
                            }
                        }
                    ).collect()).await?;

                    Ok(package)
                }
            )
        }
    ).await
}


