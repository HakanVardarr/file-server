use actix_web::{dev::Service, get, web, App, HttpResponse, HttpServer, Responder};
use diesel::query_dsl::methods::FilterDsl;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, RunQueryDsl, SqliteConnection};
use file_server::models::{NewUser, User};

use file_server::schema::users::dsl::users;
use file_server::schema::users::name;

const PORT: u16 = 8080;
type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/user/{name}")]
async fn create_new_user(pool: web::Data<DbPool>, user_name: web::Path<String>) -> impl Responder {
    let user_name = user_name.into_inner();

    let user: Result<Result<User, &str>, actix_web::error::BlockingError> = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        let new_user = NewUser { name: user_name };

        match diesel::insert_into(users)
            .values(&new_user)
            .execute(&mut conn)
        {
            Ok(_) => {
                return Ok(users
                    .filter(name.eq(&new_user.name))
                    .first::<User>(&mut conn)
                    .expect("Error retrieving inserted user"))
            }
            Err(_) => return Err("User Already Exists"),
        }
    })
    .await;

    if let Ok(user_result) = user {
        if let Ok(user) = user_result {
            HttpResponse::Ok().json(user)
        } else {
            HttpResponse::BadRequest().finish()
        }
    } else {
        HttpResponse::InternalServerError().body("Something went wrong on server")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABSE_URL NOT FOUND");
    let database_pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap_fn(|req, srv| {
                let method = req.method();
                let path = req.path();

                log::info!("{method} {path}");

                srv.call(req)
            })
            .app_data(web::Data::new(database_pool.clone()))
            .service(index)
            .service(create_new_user)
    })
    .bind(("127.0.0.1", PORT))?
    .workers(4)
    .run()
    .await?;

    Ok(())
}
