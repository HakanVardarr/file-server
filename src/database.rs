use diesel::query_dsl::methods::FilterDsl;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, RunQueryDsl, SqliteConnection};

use std::env;

use thiserror::Error;

use crate::models::{NewUser, User};
use crate::schema::users::dsl::users;
use crate::schema::users::username;
use crate::DatabasePool;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("There is no database url in the enviroment.")]
    NoUrl,
    #[error("Pool is unable to open minumum connections.")]
    PoolMinumumConnection,
    #[error("Connection timeout.")]
    ConnectionTimeout,
    #[error("Username exists choose another one.")]
    UserameExists,
    #[error("Email exists choose another one.")]
    EmailExists,
}

#[derive(Clone, Debug)]
pub struct Database {
    pool: DatabasePool,
}

impl Database {
    pub fn new() -> Result<Self, DatabaseError> {
        let url = env::var("DATABASE_URL").map_err(|_| DatabaseError::NoUrl)?;
        let pool = Pool::builder()
            .build(ConnectionManager::<SqliteConnection>::new(url))
            .map_err(|_| DatabaseError::PoolMinumumConnection)?;

        Ok(Self { pool })
    }
    pub fn insert_user(&self, new_user: NewUser) -> Result<User, DatabaseError> {
        let mut connection = self
            .pool
            .get()
            .map_err(|_| DatabaseError::ConnectionTimeout)?;

        diesel::insert_into(users)
            .values(&new_user)
            .execute(&mut connection)
            .map_err(|e| {
                let error_string = e.to_string();
                let field_part = error_string.split_once(":").unwrap().1;
                let field = field_part.split_once(".").unwrap().1;

                if field == "username" {
                    DatabaseError::UserameExists
                } else {
                    DatabaseError::EmailExists
                }
            })?;

        let user = users
            .filter(username.eq(&new_user.username))
            .first::<User>(&mut connection)
            .unwrap();

        Ok(user)
    }
}
