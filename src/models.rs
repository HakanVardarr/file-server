use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithoutPassword {
    pub username: String,
    pub email: String,
    pub api_key: String,
}

#[derive(Insertable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Insertable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUserWithApiKey {
    pub username: String,
    pub email: String,
    pub password: String,
    pub api_key: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub pasword: String,
    pub api_key: String,
}
