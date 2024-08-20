

diesel_ext --model > newmodels -d Queryable,Selectable,Debug,Clone,Insertable -t -r -I "diesel::{data_types::*, prelude::*}" -I "crate::schema::*"  --map "Interval PgInterval" --map "Timestamp PgTimestamp"