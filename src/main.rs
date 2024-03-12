use actix_web::{
    dev::Service, get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use std::io::{BufReader, Read, Write};

const PORT: u16 = 8080;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/echo/{path:.*}")]
async fn echo(path: web::Path<String>) -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/plain"))
        .body(path.into_inner())
}

#[get("/user-agent")]
async fn user_agent(req: HttpRequest) -> impl Responder {
    if let Some(user_agent) = req.headers().get("user-agent") {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(format!("User-Agent: {}", user_agent.to_str().unwrap()))
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/files/{file_name}")]
async fn get_file(file_name: web::Path<String>) -> impl Responder {
    let file_name = file_name.into_inner();

    if let Ok(file) = std::fs::File::open(&file_name) {
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        if !reader.read_to_string(&mut buffer).is_err() {
            return HttpResponse::Ok()
                .content_type("application/octet-stream")
                .body(buffer);
        }
    }

    HttpResponse::NotFound()
        .content_type("text/plain")
        .body(format!(
            "Cannot locate file called: {file_name}\nIn directory: ."
        ))
}

#[post("/files/{file_name}")]
async fn post_file(file_name: web::Path<String>, body: web::Bytes) -> impl Responder {
    let file_name = file_name.into_inner();

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_name)
    {
        if !file.write_all(&body).is_err() {
            return HttpResponse::Created().finish();
        }
    }
    return HttpResponse::InternalServerError().finish();
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
            .service(echo)
            .service(user_agent)
            .service(get_file)
            .service(post_file)
    })
    .bind(("127.0.0.1", PORT))?
    .workers(4)
    .run()
    .await?;

    Ok(())
}
