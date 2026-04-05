use std::{env, sync::Arc};

use poem::{handler, http::StatusCode, web::Data, Error, Response};
use serde_json::{json, Value};
use sqlx::query_as;

use crate::models::{AppState, StillRow};

#[handler]
pub async fn get_feed_json(data: Data<&Arc<AppState>>) -> Result<Response, Error> {
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "https://inert.example.com".to_string());

    let rows: Vec<StillRow> = query_as(
        r#"
        SELECT s.still_id, s.slug, s.user_id, u.username, u.display_name,
               s.caption, s.published_at,
               COALESCE(
                   json_group_array(json_object(
                       'image_id', si.image_id,
                       'image_url', si.image_url,
                       'width', si.width,
                       'height', si.height,
                       'position', si.position
                   )) FILTER (WHERE si.image_id IS NOT NULL),
                   '[]'
               ) as images_json
        FROM stills s
        JOIN users u ON s.user_id = u.user_id
        LEFT JOIN still_images si ON s.still_id = si.still_id
        GROUP BY s.still_id
        ORDER BY s.published_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        Error::from_string(format!("피드 조회 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let stills: Vec<_> = rows.into_iter().map(StillRow::into_response).collect();

    let items: Vec<Value> = stills
        .iter()
        .map(|s| {
            let first_image_url = s.images.first().map(|i| i.image_url.as_str()).unwrap_or("");
            json!({
                "id": format!("{}/s/{}", base_url, s.slug),
                "url": format!("{}/s/{}", base_url, s.slug),
                "image": first_image_url,
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
