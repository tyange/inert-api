mod db;
mod middlewares;
mod models;
mod routes;

use std::{env, fs, sync::Arc};

use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Region;
use db::init_db;
use dotenv::dotenv;
use middlewares::auth_middleware::Auth;
use models::AppState;
use poem::{
    delete, get,
    handler,
    http::StatusCode,
    listener::TcpListener,
    middleware::{Cors, SizeLimit},
    options, post,
    EndpointExt, Response, Route, Server,
};
use routes::{
    create_still::create_still,
    delete_still::delete_still,
    get_feed::get_feed,
    get_feed_json::get_feed_json,
    get_feed_rss::get_feed_rss,
    get_mine::get_mine,
    get_still::get_still,
    get_user_stills::get_user_stills,
    login::login,
    login_google::login_google,
    me::me,
    signup::signup,
    upload_image::upload_image,
};
use sqlx::SqlitePool;

const DEFAULT_UPLOAD_MAX_BYTES: usize = 20 * 1024 * 1024;

#[handler]
fn health() -> &'static str {
    "ok"
}

#[handler]
async fn options_handler() -> Response {
    Response::builder().status(StatusCode::OK).finish()
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/inert.db".to_string());
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let db_url = format!("sqlite:{}?mode=rwc", db_path);
    let db = SqlitePool::connect(&db_url).await.map_err(|e| {
        eprintln!("DB 연결 실패: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;

    init_db(&db)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // S3 클라이언트 (Lightsail Object Storage)
    let endpoint_url =
        env::var("AWS_ENDPOINT_URL").unwrap_or_else(|_| "https://s3.amazonaws.com".to_string());
    let region = env::var("AWS_REGION").unwrap_or_else(|_| "ap-northeast-1".to_string());
    let bucket_name = env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "inert".to_string());
    let cdn_base_url =
        env::var("CDN_BASE_URL").unwrap_or_else(|_| "https://cdn.example.com".to_string());

    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(&endpoint_url)
        .region(Region::new(region))
        .load()
        .await;
    let s3 = aws_sdk_s3::Client::new(&aws_config);

    let state = Arc::new(AppState { db, s3, bucket_name, cdn_base_url });

    let upload_max_bytes = env::var("UPLOAD_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_UPLOAD_MAX_BYTES);

    let allowed_origins: Vec<String> = env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let mut cors = Cors::new()
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_credentials(true)
        .allow_headers(vec!["authorization", "content-type", "accept"]);

    for origin in &allowed_origins {
        cors = cors.allow_origin(origin.as_str());
    }

    let app = Route::new()
        // 헬스체크
        .at("/health", get(health))
        // 인증
        .at("/auth/signup", post(signup))
        .at("/auth/login", post(login))
        .at("/auth/login/google", post(login_google))
        .at("/auth/me", get(me).with(Auth))
        // Stills
        .at("/stills", post(create_still).with(Auth))
        .at("/stills/mine", get(get_mine).with(Auth))
        .at("/stills/:still_id", delete(delete_still).with(Auth))
        // 이미지 업로드
        .at(
            "/images/upload",
            post(upload_image)
                .with(SizeLimit::new(upload_max_bytes))
                .with(Auth),
        )
        // 공개 피드
        .at("/feed", get(get_feed))
        .at("/feed.rss", get(get_feed_rss))
        .at("/feed.json", get(get_feed_json))
        // 공개 still
        .at("/s/:slug", get(get_still))
        // 유저 공개 페이지
        .at("/u/:username", get(get_user_stills))
        // CORS preflight
        .at("/*path", options(options_handler))
        .data(state)
        .with(cors);

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    println!("inert-api 시작: http://0.0.0.0:{}", port);

    Server::new(TcpListener::bind(format!("0.0.0.0:{}", port)))
        .run(app)
        .await
}
