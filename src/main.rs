use actix_web::{web::Data, App, HttpServer};
use env_logger::Env;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod handlers;
mod models;

#[derive(Clone)]
pub struct AppState {
    db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("Trying to connect to the database: {}", &database_url);
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Error building a connection pool");
    let state = AppState { db: pool };
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .configure(handlers::init_routes)
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("0.0.0.0", 9999))?
    .run()
    .await
}
