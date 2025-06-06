use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use std::path::PathBuf;

struct Stats {
    cpu_cores: u8,
    cpu_usage: f32,
    total_memory: u64,
    used_memory: u64,

}

#[get("/")]
async fn index() -> actix_web::Result<NamedFile> {
    let path: PathBuf = PathBuf::from(r".\src\pages\index.html");
    Ok(NamedFile::open(path)?)
}

#[get("/stats")]
async fn stats() -> impl Responder {
    HttpResponse::Ok().body("stats")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(stats)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
