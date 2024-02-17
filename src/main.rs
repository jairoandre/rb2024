use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use env_logger::Env;
use serde::Serialize;
#[macro_use]
extern crate diesel_migrations;

mod handlers;
mod models;
mod repository;

#[derive(Serialize)]
pub struct Response {
    status: String,
    message: String,
}

type DB = diesel::pg::Pg;
const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(connection: &mut impl MigrationHarness<DB>) {
    let _ = connection.run_pending_migrations(MIGRATIONS);
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(Response {
        status: "ok".to_string(),
        message: "Server is running".to_string(),
    })
}

async fn not_found_error() -> Result<HttpResponse> {
    Ok(HttpResponse::NotFound().json(Response {
        status: "error".to_string(),
        message: "Not Found".to_string(),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let rinha_db = repository::db::Database::new();
    run_migrations(&mut rinha_db.pool.get().unwrap());
    let app_data = web::Data::new(rinha_db);
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .configure(handlers::handlers::init_routes)
            .service(health)
            .default_service(web::route().to(not_found_error))
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
