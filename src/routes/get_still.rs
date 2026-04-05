use std::sync::Arc;

use poem::{handler, http::StatusCode, web::{Data, Json, Path}, Error};
use sqlx::query_as;

use crate::models::{AppState, CustomResponse, StillResponse, StillRow};

#[handler]
pub async fn get_still(
    Path(slug): Path<String>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<StillResponse>>, Error> {
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
        WHERE s.slug = ?
        GROUP BY s.still_id
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

    Ok(Json(CustomResponse::ok(row.into_response())))
}
