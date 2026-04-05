use std::{env, sync::Arc};

use poem::{handler, http::StatusCode, web::Data, Error, Response};
use serde_json::{json, Value};
use sqlx::query_as;

use crate::models::{AppState, StillResponse};

#[handler]
pub async fn get_feed_json(data: Data<&Arc<AppState>>) -> Result<Response, Error> {
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "https://inert.example.com".to_string());

    let stills: Vec<StillResponse> = query_as(
        r#"
        SELECT s.still_id, s.slug, s.user_id, u.username, u.display_name,
               s.caption, s.image_url, s.width, s.height, s.published_at
        FROM stills s JOIN users u ON s.user_id = u.user_id
        ORDER BY s.published_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        Error::from_string(format!("피드 조회 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let items: Vec<Value> = stills
        .iter()
        .map(|s| {
            json!({
                "id": format!("{}/s/{}", base_url, s.slug),
                "url": format!("{}/s/{}", base_url, s.slug),
                "image": s.image_url,
                "content_text": s.caption,
                "date_published": s.published_at,
                "authors": [{ "name": s.display_name.as_deref().unwrap_or(&s.username) }],
            })
        })
        .collect();

    let feed = json!({
        "version": "https://jsonfeed.org/version/1.1",
        "title": "inert",
        "home_page_url": base_url,
        "feed_url": format!("{}/feed.json", base_url),
        "description": "inert — 무해한 사진과 글",
        "items": items,
    });

    Ok(Response::builder()
        .content_type("application/feed+json; charset=utf-8")
        .body(feed.to_string()))
}
