use std::{env, sync::Arc};

use poem::{handler, http::StatusCode, web::Data, Error, Response};
use rss::{ChannelBuilder, ItemBuilder};
use sqlx::query_as;

use crate::models::{AppState, StillRow};

#[handler]
pub async fn get_feed_rss(data: Data<&Arc<AppState>>) -> Result<Response, Error> {
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

    let items: Vec<rss::Item> = stills
        .iter()
        .map(|s| {
            let url = format!("{}/s/{}", base_url, s.slug);
            let first_image_url = s.images.first().map(|i| i.image_url.as_str()).unwrap_or("");
            let description = format!(
                r#"<img src="{}" alt="still" /><p>{}</p>"#,
                first_image_url,
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
