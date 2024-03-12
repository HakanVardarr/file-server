use file_server::run;

#[actix_web::main]
async fn main() {
    match run().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("[ERROR]: {e}");
            std::process::exit(1);
        }
    }
}
