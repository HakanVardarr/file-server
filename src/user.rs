use super::{
    database::{Database, DatabaseError},
    models::{NewUser, UserWithoutPassword},
};
use actix_web::{post, web, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use thiserror::Error;

#[derive(Debug, Error)]
enum UserError {
    #[error("Unable to hash the password.")]
    HashError,
}

fn hash_user(new_user: &NewUser) -> Result<NewUser, UserError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(&new_user.password.as_bytes(), &salt)
        .map_err(|_| UserError::HashError)?
        .to_string();

    Ok(NewUser {
        username: new_user.username.clone(),
        email: new_user.email.clone(),
        password: password_hash,
    })
}

#[post("/user/register")]
async fn register(database: web::Data<Database>, new_user: web::Json<NewUser>) -> impl Responder {
    let hashed_user = hash_user(&new_user);

    if hashed_user.is_err() {
        return HttpResponse::InternalServerError().body("Unable to hash the password.");
    }

    let user = web::block(move || database.insert_user(hashed_user.unwrap())).await;

    if user.is_ok() {
        return match user.unwrap() {
            Ok((user, key)) => HttpResponse::Created().json(UserWithoutPassword {
                username: user.username,
                email: user.email,
                api_key: key,
            }),
            Err(e) => match e {
                DatabaseError::UserameExists => HttpResponse::Conflict()
                    .body(format!("This username exists: {}", new_user.username)),
                DatabaseError::EmailExists => {
                    HttpResponse::Conflict().body(format!("This email exists: {}", new_user.email))
                }
                _ => HttpResponse::InternalServerError().body(e.to_string()),
            },
        };
    }

    HttpResponse::InternalServerError().finish()
}
