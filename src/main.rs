pub mod models;
pub mod routes;
use actix_files::Files;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;
use actix_web::{
    dev::Server, get, http::StatusCode, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use std::sync::Mutex;
use std::{env, fs::read_to_string};
use tera::Tera;

pub struct AppState {
    js_source: Mutex<String>, // <- Mutex is necessary to mutate safely across threads
}

fn get_secret_key() -> Result<Key, Box<dyn std::error::Error>> {
    let secret_key_from_env = env::var("SECRET_KEY")?;
    if secret_key_from_env.len() < 32 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Secret key must be at least 32 characters",
        )));
    }
    let key = Key::from(secret_key_from_env.as_bytes());
    Ok(key)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("Starting Actix web server...");

    HttpServer::new(move || {
        // Load tera templates from the specified directory
        println!("Initializing Actix web application...");

        let state = web::Data::new(AppState {
            js_source: Mutex::new(
                read_to_string("./build/index.js").expect("Failed to load the resource."),
            ),
        });

        App::new()
            .app_data(state)
            .wrap(
                actix_web::middleware::Logger::default()
                    .exclude("/static")
                    .exclude("/favicon.ico"),
            )
            .wrap(actix_session::SessionMiddleware::new(
                CookieSessionStore::default(),
                get_secret_key().expect("Failed to generate secret key"),
            ))
            .service(Files::new("/static", "./static").show_files_listing())
            .service(Files::new("/react-ssr", "./client/build").show_files_listing())
            .service(Files::new("/scripts", "./build/scripts/").show_files_listing())
            .route("/health_check", web::get().to(health_check))
            .service(routes::index::index)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}