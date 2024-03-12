use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub name: String,
}
