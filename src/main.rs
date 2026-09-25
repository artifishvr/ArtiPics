use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use moka::future::Cache;
use rayon::prelude::*;
use std::time::Duration;

mod analytics;
mod helpers;
mod reqwest_client;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Clone)]
struct AppState {
    cache: Cache<String, String>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cache = Cache::builder()
        .time_to_live(Duration::from_hours(3))
        .max_capacity(10)
        .build();

    let state = AppState { cache };

    let app = Router::new()
        .route("/", get(home_redirect))
        .route("/pics", get(pics_route))
        .route("/meow", get("meow"))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("\n---------------\nrunning - 0.0.0.0:3000\n---------------\n");
    axum::serve(listener, app).await.unwrap();
}

async fn pics_route(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    tokio::spawn(async move {
        let _ = analytics::yell("/pics", headers).await;
    });

    match state.cache.get("pics").await {
        Some(s) => {
            println!("cache HIT");
            return format!("success-cache,{}", s);
        }
        None => println!("cache MISS"),
    }

    let pics = match helpers::get_images().await {
        Ok(s) => s,
        Err(e) => return format!("error getting images: {}", e),
    };

    if pics.len() < 1 {
        return format!("no images");
    }

    let bench = std::time::Instant::now();

    let base64string = pics
        .par_iter()
        .map( |image_url| {
            return helpers::download_image_to_rgb_b64(image_url, 171, 256).unwrap();
        })
        .collect::<Vec<_>>()
        .join(",");

    println!("pic construction took {:?}", bench.elapsed());

    state.cache.insert("pics".to_string(), base64string.clone()).await;

    format!("success,{}", base64string)
}

async fn home_redirect(headers: HeaderMap) -> impl IntoResponse {
    tokio::spawn(async move {
        let _ = analytics::yell("/", headers).await;
    });

    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "https://pics.arti.gay")],
    )
}