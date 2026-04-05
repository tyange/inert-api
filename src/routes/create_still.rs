use std::sync::Arc;

use inert_api::auth::authorization::current_user;
use poem::{handler, http::StatusCode, web::{Data, Json}, Error, Request};
use sqlx::{query, query_as};
use uuid::Uuid;

use crate::models::{AppState, CreateStillRequest, CustomResponse, StillResponse, StillRow};

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

    if body.image_keys.is_empty() {
        return Err(Error::from_string("image_keys가 필요합니다.", StatusCode::BAD_REQUEST));
    }

    let still_id = Uuid::new_v4().to_string();
    let slug = generate_slug();

    query(
        r#"
        INSERT INTO stills (still_id, slug, user_id, caption)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&still_id)
    .bind(&slug)
    .bind(&user.user_id)
    .bind(&body.caption)
    .execute(&data.db)
    .await
    .map_err(|e| {
        Error::from_string(format!("still 생성 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    for (position, image_key) in body.image_keys.iter().enumerate() {
        if image_key.is_empty() {
            continue;
        }
        let image_url = format!("{}/{}", data.cdn_base_url.trim_end_matches('/'), image_key);
        let image_id = Uuid::new_v4().to_string();

        query(
            r#"
            INSERT INTO still_images (image_id, still_id, image_url, image_key, position)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&image_id)
        .bind(&still_id)
        .bind(&image_url)
        .bind(image_key)
        .bind(position as i64)
        .execute(&data.db)
        .await
        .map_err(|e| {
            Error::from_string(format!("이미지 저장 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
    }

    let row: StillRow = query_as(
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
        WHERE s.still_id = ?
        GROUP BY s.still_id
        "#,
    )
    .bind(&still_id)
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        Error::from_string(format!("still 조회 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    Ok(Json(CustomResponse::ok_msg(row.into_response(), "still이 생성되었습니다.")))
}
