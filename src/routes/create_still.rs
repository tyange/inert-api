use std::sync::Arc;

use inert_api::auth::authorization::current_user;
use poem::{handler, http::StatusCode, web::{Data, Json}, Error, Request};
use sqlx::{query, query_as};
use uuid::Uuid;

use crate::models::{AppState, CreateStillRequest, CustomResponse, StillResponse};

fn generate_slug() -> String {
    let alphabet: Vec<char> = "0123456789abcdefghijklmnopqrstuvwxyz".chars().collect();
    nanoid::nanoid!(8, &alphabet)
}

#[handler]
pub async fn create_still(
    req: &Request,
    Json(body): Json<CreateStillRequest>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<StillResponse>>, Error> {
    let user = current_user(req)?;

    if body.image_key.is_empty() {
        return Err(Error::from_string("image_key가 필요합니다.", StatusCode::BAD_REQUEST));
    }

    let image_url = format!("{}/{}", data.cdn_base_url.trim_end_matches('/'), body.image_key);
    let still_id = Uuid::new_v4().to_string();
    let slug = generate_slug();

    query(
        r#"
        INSERT INTO stills (still_id, slug, user_id, caption, image_url, image_key)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&still_id)
    .bind(&slug)
    .bind(&user.user_id)
    .bind(&body.caption)
    .bind(&image_url)
    .bind(&body.image_key)
    .execute(&data.db)
    .await
    .map_err(|e| {
        Error::from_string(format!("still 생성 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let still: StillResponse = query_as(
        r#"
        SELECT s.still_id, s.slug, s.user_id, u.username, u.display_name,
               s.caption, s.image_url, s.width, s.height, s.published_at
        FROM stills s JOIN users u ON s.user_id = u.user_id
        WHERE s.still_id = ?
        "#,
    )
    .bind(&still_id)
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        Error::from_string(format!("still 조회 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    Ok(Json(CustomResponse::ok_msg(still, "still이 생성되었습니다.")))
}
