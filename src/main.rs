mod db;
mod errors;
mod filters;
mod handlers;

use actix_files::Files;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use tera::Tera;

use crate::db::build_pool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16");
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "cwapp.db".to_string());

    let pool = build_pool(&database_url).expect("Failed to initialize database pool");

    let mut tera = Tera::new("templates/**/*").expect("Failed to load templates");
    tera.register_filter("icon", filters::icon_filter);

    let pool_data = web::Data::new(pool);
    let tmpl_data = web::Data::new(tera);

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .app_data(tmpl_data.clone())
            .wrap(Logger::default())
            .service(handlers::index)
            .service(handlers::create_job)
            .service(handlers::list_jobs)
            .service(handlers::get_job)
            .service(handlers::update_status)
            .service(handlers::delete_job)
            .service(handlers::get_stats)
            .service(handlers::get_earnings)
            .service(Files::new("/static", "static"))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
