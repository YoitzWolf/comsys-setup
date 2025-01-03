use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::auth_backend::tokens::Permissions;
use crate::models::{User, UserOrg};

#[derive(Queryable,Selectable,Insertable,Debug,Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct UserModel {
    pub login: String,
    pub selfname: String,
    pub hash: String,
}

pub async fn get_by_login(conn: &mut AsyncPgConnection, login: &str) -> Result<User, Error> {
    use crate::schema::users::dsl as dsl;
    dsl::users.filter(dsl::login.eq(login)).select(User::as_select()).first(conn).await
}


pub async fn get_by_id(conn: &mut AsyncPgConnection, id: i32) -> Result<User, Error> {
    use crate::schema::users::dsl as dsl;
    dsl::users.filter(dsl::id.eq(id)).select(User::as_select()).first(conn).await
}

pub async fn setup_selfname(conn: &mut AsyncPgConnection, id: i32, selfname: &str) -> Result<usize, Error> {
    use crate::schema::users::dsl as dsl;
    diesel::update( dsl::users)
        .filter(dsl::id.eq(id))
        .set(dsl::selfname.eq(selfname)).execute(conn).await
}

/*
pub async fn get_user_orgs(conn: &mut AsyncPgConnection, id: i32) -> Result<Vec<UserOrg>, Error> {
    use crate::schema::user_orgs::dsl as dsl;
    dsl::user_orgs.filter(dsl::uid.eq(id)).select(UserOrg::as_select()).get_results(conn).await
}


pub async fn get_user_perms(conn: &mut AsyncPgConnection, id: i32) -> Result<Vec<(i32, Vec<Permissions>)>, Error> {
    match get_user_orgs(conn, id).await {
        Ok(x) => {
            Ok(x.iter().map(|v| {(v.oid, Permissions::parse(&v.perm).unwrap_or_default())}).collect())
        }
        Err(e) => Err(e)
    }
}*/

pub async fn insert_users(conn: &mut AsyncPgConnection, users: &Vec<UserModel>) -> Result<Vec<User>, Error> {
    use crate::schema::users::dsl as dsl;
    diesel::insert_into(dsl::users).values(users).get_results(conn).await
}

pub async fn insert_user(conn: &mut AsyncPgConnection, user: &UserModel) -> Result<Vec<User>, Error> {
    use crate::schema::users::dsl as dsl;
    diesel::insert_into(dsl::users).values(user).get_results(conn).await
}
