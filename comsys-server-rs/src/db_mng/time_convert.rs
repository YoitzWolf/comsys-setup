use chrono::{DateTime, Datelike, NaiveDateTime};
use diesel::{data_types::PgTimestamp, deserialize::FromSql};
use prost_wkt_types::Timestamp;
use tracing::{info, instrument};

pub fn into_naive(t: Option::<Timestamp>) -> Option<NaiveDateTime> {
    if let Some(t) = t {
        Some(DateTime::from_timestamp(t.seconds, 0).unwrap().naive_utc())
    } else {
        None
    }
}

pub fn into_timestamp(t: Option::<NaiveDateTime>) -> Option<Timestamp> {

    if let Some(t) = t {
        Some(t.into())
    } else {
        None
    }
}

#[instrument]
pub fn pg_into_naive(t: PgTimestamp) -> NaiveDateTime {
    //NaiveDateTime::from_sql(t).unwrap()
    todo!()
}

pub fn pg_into_timestamp(t: Option::<PgTimestamp>) -> Option<Timestamp> {

    if let Some(t) = t {
        Some(pg_into_naive(t).into())
    } else {
        None
    }
}