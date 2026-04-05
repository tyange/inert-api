use std::sync::Arc;

use poem::{handler, http::StatusCode, web::{Data, Json, Path}, Error};
use sqlx::query_as;

use crate::models::{AppState, CustomResponse, StillResponse};

#[handler]
pub async fn get_still(
    Path(slug): Path<String>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<StillResponse>>, Error> {
    let still: StillResponse = query_as(
        r#"
        SELECT s.still_id, s.slug, s.user_id, u.username, u.display_name,
               s.caption, s.image_url, s.width, s.height, s.published_at
        FROM stills s JOIN users u ON s.user_id = u.user_id
        WHERE s.slug = ?
        "#,
    )
    .bind(&slug)
    .fetch_one(&data.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            Error::from_string("still을 찾을 수 없습니다.", StatusCode::NOT_FOUND)
        }
        _ => Error::from_string(
            format!("조회 실패: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    })?;

    Ok(Json(CustomResponse::ok(still)))
}
