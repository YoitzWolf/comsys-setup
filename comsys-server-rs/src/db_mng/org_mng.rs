use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use crate::auth_backend::tokens::Permissions;
use crate::models::{CompStaffLink, Organisation, User, UserOrg};

pub async fn get_user_orgs_by_uid(conn: &mut AsyncPgConnection, uid: i32) -> Result<Vec<UserOrg>, Error> {
    use crate::schema::user_orgs::dsl;
    dsl::user_orgs.filter(dsl::uid.eq(uid)).select(UserOrg::as_select()).get_results(conn).await
}
pub async fn get_ownerships(conn: &mut AsyncPgConnection, uid: i32) -> Result<Vec<Organisation>, Error> {
    use crate::schema::organisations::dsl;
    dsl::organisations.filter(dsl::owner.eq(uid)).select(Organisation::as_select()).get_results(conn).await
}
pub async fn get_ownership(conn: &mut AsyncPgConnection, oid: i32) -> Result<Organisation, Error> {
    use crate::schema::organisations::dsl;
    dsl::organisations.filter(dsl::id.eq(oid)).select(Organisation::as_select()).first(conn).await
}
pub async fn is_owner_of(conn: &mut AsyncPgConnection, uid: i32, oid: i32) -> Result<Organisation, Error> {
    use crate::schema::organisations::dsl;
    dsl::organisations.filter(dsl::id.eq(oid).and(dsl::owner.eq(uid))).select(Organisation::as_select()).first(conn).await
}
pub async fn setup_org_perms_to_user(
    conn: &mut AsyncPgConnection,
    uid: i32,
    oid: i32,
    perms: &Vec<Permissions>
) -> Result<usize, Error> {
    use crate::schema::user_orgs::dsl;
    diesel::insert_into(dsl::user_orgs).values(
        perms.iter().map(|x| {
            UserOrg {
                uid,
                oid,
                perm: serde_json::to_string(x).unwrap(),
            }
        }).collect::<Vec<UserOrg>>()
    ).execute(conn).await
}

pub async fn create_com_staff(
    conn: &mut AsyncPgConnection,
    vals: &Vec<CompStaffLink>
) -> Result<usize, Error> {
    use crate::schema::comp_staff_links::dsl;
    diesel::insert_into(dsl::comp_staff_links).values(
        vals
    ).execute(conn).await
}