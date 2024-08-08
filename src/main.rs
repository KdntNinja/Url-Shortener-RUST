use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use uuid::Uuid;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use dotenv::dotenv;
use std::env;
use std::fs::File;
use log::error;
use env_logger;
use env_logger::init;

struct AppState {
    db_pool: SqlitePool,
}

async fn insert_url(db_pool: &SqlitePool, shortened_url: &str, original_url: &str) -> Result<(), sqlx::Error> {
    let query = "INSERT INTO url_map (shortened_url, original_url) VALUES (?, ?)";
    sqlx::query(query)
        .bind(shortened_url)
        .bind(original_url)
        .execute(db_pool)
        .await?;
    Ok(())
}

async fn get_shortened_url(db_pool: &SqlitePool, original_url: &str) -> Result<Option<String>, sqlx::Error> {
    let query = "SELECT shortened_url FROM url_map WHERE original_url = ?";
    let result: Option<(String,)> = sqlx::query_as(query)
        .bind(original_url)
        .fetch_optional(db_pool)
        .await?;
    Ok(result.map(|(shortened_url,)| shortened_url))
}

async fn respond_with_insert_result(db_pool: &SqlitePool, shortened_url: &str, original_url: &str) -> HttpResponse {
    match insert_url(db_pool, shortened_url, original_url).await {
        Ok(_) => HttpResponse::Ok().body(shortened_url.to_string()),
        Err(e) => {
            error!("Failed to insert URL: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

async fn shorten_url(data: web::Data<AppState>, req_body: String) -> impl Responder {
    if let Ok(Some(existing_shortened_url)) = get_shortened_url(&data.db_pool, &req_body).await {
        return HttpResponse::Ok().body(existing_shortened_url);
    }
    let shortened_url = Uuid::new_v4().to_string()[..8].to_string();
    respond_with_insert_result(&data.db_pool, &shortened_url, &req_body).await
}

async fn custom_shorten_url(data: web::Data<AppState>, path: web::Path<(String,)>, req_body: String) -> impl Responder {
    if let Ok(Some(existing_shortened_url)) = get_shortened_url(&data.db_pool, &req_body).await {
        return HttpResponse::Ok().body(existing_shortened_url);
    }
    let custom_shortened_url = path.into_inner().0;
    let query_check = "SELECT COUNT(*) FROM url_map WHERE shortened_url = ?";
    let count: (i64,) = sqlx::query_as(query_check)
        .bind(&custom_shortened_url)
        .fetch_one(&data.db_pool)
        .await
        .unwrap();
    if count.0 > 0 {
        HttpResponse::Conflict().body("Custom URL already exists")
    } else {
        respond_with_insert_result(&data.db_pool, &custom_shortened_url, &req_body).await
    }
}

async fn redirect_to_original(data: web::Data<AppState>, shortened_url: web::Path<String>) -> impl Responder {
    let query = "SELECT original_url FROM url_map WHERE shortened_url = ?";
    let result: Option<(String,)> = sqlx::query_as(query)
        .bind(&shortened_url.into_inner())
        .fetch_optional(&data.db_pool)
        .await
        .unwrap();
    if let Some((original_url,)) = result {
        HttpResponse::Found().append_header(("Location", original_url)).finish()
    } else {
        HttpResponse::NotFound().body("URL not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    init();
    let mut database_path = env::current_dir().expect("Failed to get current directory");
    database_path.push("src/database.db");

    if !database_path.exists() {
        File::create(&database_path).expect("Failed to create database file");
    }

    let database_url = database_path.to_str().expect("Failed to convert path to string");

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to create pool.");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS url_map (
            id INTEGER PRIMARY KEY,
            shortened_url TEXT NOT NULL,
            original_url TEXT NOT NULL
        )"
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create table.");

    let app_state = web::Data::new(AppState { db_pool });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/shorten", web::post().to(shorten_url))
            .route("/{custom}/shorten", web::post().to(custom_shorten_url))
            .route("/{shortened_url}", web::get().to(redirect_to_original))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}