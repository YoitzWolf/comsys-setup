use diesel::prelude::*;

use std::sync::Arc;
use diesel::result::Error;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use tokio::sync::Mutex;
use crate::models::User;

use crate::schema::users as users;
use crate::schema::users::dsl as dsl;

pub async fn get_by_login(mut conn: &mut AsyncPgConnection, login: &str) -> Result<User, Error> {
    dsl::users.filter(dsl::login.eq(login)).select(User::as_select()).first(conn).await
}


pub async fn get_by_id(mut conn: &mut AsyncPgConnection, id: i32) -> Result<User, Error> {
    dsl::users.filter(dsl::id.eq(id)).select(User::as_select()).first(conn).await
}
