use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use diesel::query_dsl::methods::FilterDsl;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{ExpressionMethods, RunQueryDsl, SqliteConnection};
use rand::Rng;

use std::env;

use thiserror::Error;

use crate::models::{NewUser, NewUserWithApiKey, User};
use crate::schema::users::dsl::users;
use crate::schema::users::{api_key, username};
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
    #[error("Unable to hash api key.")]
    HashError,
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
    pub fn insert_user(&self, new_user: NewUser) -> Result<(User, String), DatabaseError> {
        let mut connection = self
            .pool
            .get()
            .map_err(|_| DatabaseError::ConnectionTimeout)?;

        let key = generate_api_key(&mut connection);
        let hashed_key = hash_api_key(&key)?;

        let new_user = NewUserWithApiKey {
            username: new_user.username,
            email: new_user.email,
            password: new_user.password,
            api_key: hashed_key,
        };

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

        Ok((user, key))
    }
}

fn generate_api_key(
    connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> String {
    let mut rng = rand::thread_rng();

    let key: String = (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();

    let result = users.filter(api_key.eq(&key)).first::<User>(connection);

    match result {
        Ok(_) => return generate_api_key(connection),
        Err(_) => key,
    }
}

fn hash_api_key(key: &String) -> Result<String, DatabaseError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(&key.as_bytes(), &salt)
        .map_err(|_| DatabaseError::HashError)?
        .to_string();

    Ok(password_hash)
}
