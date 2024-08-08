use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Mutex;

struct AppState {
    url_map: Mutex<HashMap<String, String>>,
}

async fn shorten_url(data: web::Data<AppState>, req_body: String) -> impl Responder {
    let mut url_map = data.url_map.lock().unwrap();
    let shortened_url = Uuid::new_v4().to_string();
    url_map.insert(shortened_url.clone(), req_body);
    HttpResponse::Ok().body(shortened_url)
}

async fn redirect_to_original(data: web::Data<AppState>, shortened_url: web::Path<String>) -> impl Responder {
    let url_map = data.url_map.lock().unwrap();
    if let Some(original_url) = url_map.get(&shortened_url.into_inner()) {
        HttpResponse::Found().header("Location", original_url.clone()).finish()
    } else {
        HttpResponse::NotFound().body("URL not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        url_map: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/shorten", web::post().to(shorten_url))
            .route("/{shortened_url}", web::get().to(redirect_to_original))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}