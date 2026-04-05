use std::{env, sync::Arc};

use poem::{handler, http::StatusCode, web::Data, Error, Response};
use rss::{ChannelBuilder, ItemBuilder};
use sqlx::query_as;

use crate::models::{AppState, StillResponse};

#[handler]
pub async fn get_feed_rss(data: Data<&Arc<AppState>>) -> Result<Response, Error> {
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

    let items: Vec<rss::Item> = stills
        .iter()
        .map(|s| {
            let url = format!("{}/s/{}", base_url, s.slug);
            let description = format!(
                r#"<img src="{}" alt="still" /><p>{}</p>"#,
                s.image_url,
                s.caption.as_deref().unwrap_or("")
            );
            ItemBuilder::default()
                .title(s.caption.clone().or_else(|| Some(s.slug.clone())))
                .link(Some(url))
                .description(Some(description))
                .pub_date(Some(s.published_at.clone()))
                .build()
        })
        .collect();

    let channel = ChannelBuilder::default()
        .title("inert")
        .link(base_url.clone())
        .description("inert — 무해한 사진과 글")
        .items(items)
        .build();

    Ok(Response::builder()
        .content_type("application/rss+xml; charset=utf-8")
        .body(channel.to_string()))
}
