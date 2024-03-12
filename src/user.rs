use super::{
    database::{Database, DatabaseError},
    models::{NewUser, UserWithoutPassword},
};
use actix_web::{post, web, HttpResponse, Responder};

#[post("/user/register")]
async fn register(database: web::Data<Database>, new_user: web::Json<NewUser>) -> impl Responder {
    let new_user_clone = new_user.clone();
    let user = web::block(move || database.insert_user(new_user_clone)).await;

    if user.is_ok() {
        return match user.unwrap() {
            Ok(_) => HttpResponse::Created().json(UserWithoutPassword {
                username: new_user.username.clone(),
                email: new_user.email.clone(),
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
