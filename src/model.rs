use diesel::{prelude::*, sql_types::BigInt};

use crate::schema::{division_updates, divisions};

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = divisions)]
pub struct Division {
    pub id: i32,
    pub division_id: i32,
    pub discord_thread_id: i64,
}

#[derive(Insertable)]
#[diesel(table_name = divisions)]
pub struct NewDivision {
    pub division_id: i32,
    pub discord_thread_id: i64,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Division))]
#[diesel(table_name = division_updates)]
pub struct DivisionUpdate {
    pub id: i32,
    pub division_id: i32,
    pub publication_updated: String,
}

#[derive(Insertable)]
#[diesel(table_name = division_updates)]
pub struct NewDivisionUpdate<'a> {
    pub division_id: i32,
    pub publication_updated: &'a str,
}
