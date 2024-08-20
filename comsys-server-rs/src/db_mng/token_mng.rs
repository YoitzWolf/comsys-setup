use argon2::password_hash::{Salt, SaltString};
use argon2::PasswordHasher;

use diesel::data_types::{PgInterval, PgTimestamp};
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use rand_core::OsRng;
use crate::auth_backend::tokens::TokenClaim;

use crate::models::Token;

use chrono::{TimeZone, Utc};
use chrono::{DateTime, NaiveDateTime};
use diesel::sql_types::Timestamp;
use fastmurmur3::murmur3_x64_128;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::tokens)]
pub struct TokenPrototype {
    pub hash: String,
    pub ttype: i32,
    pub owner: i32,
    pub sub: String,
    pub created_at: NaiveDateTime,
    pub expire_at: NaiveDateTime,
}


impl From<&TokenClaim> for TokenPrototype {
    fn from(claim: &TokenClaim) -> Self {
        let h = fastmurmur3::hash(claim.value.as_bytes());
        Self {
            hash: format!("{:X}", h),
            ttype: claim.token_type,
            owner: claim.user_id,
            sub: claim.sub.clone(),
            created_at: DateTime::from_timestamp(claim.iat, 0).unwrap().naive_utc(),
            expire_at:  DateTime::from_timestamp(claim.exp, 0).unwrap().naive_utc()
        }
    }
}


pub async fn get_by_id(conn: &mut AsyncPgConnection, id: i32) -> Result<Token, Error> {
    crate::schema::tokens::dsl::tokens.filter(
        crate::schema::tokens::dsl::id.eq(id)
    ).select(
        Token::as_select()
    ).first(conn).await
}


pub async fn exist_such(
    conn: &mut AsyncPgConnection,
    proto: &TokenPrototype
) -> Result<Token, Error> {
    let hash = proto.hash.clone();
    let sub = proto.sub.clone();
    crate::schema::tokens::dsl::tokens.filter(
        crate::schema::tokens::dsl::hash.eq(hash)
            .and(
                crate::schema::tokens::dsl::sub.eq(sub)
            )
            .and(
                crate::schema::tokens::dsl::ttype.eq(proto.ttype)
            )
    ).select(
        Token::as_select()
    ).first(conn).await
}


pub async fn insert_token(conn: &mut AsyncPgConnection, token: TokenPrototype) -> Result<usize, Error> {
    use crate::schema::tokens::dsl::*;
    diesel::insert_into(
        tokens
    ).values(
        &token
    ).execute(conn).await
}