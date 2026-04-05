use std::sync::Arc;

use poem::{handler, http::StatusCode, web::{Data, Json, Path, Query}, Error};
use sqlx::{query_as, query_scalar};

use crate::models::{AppState, CustomResponse, PaginationQuery, StillResponse, StillsListResponse};

#[handler]
pub async fn get_user_stills(
    Path(username): Path<String>,
    Query(params): Query<PaginationQuery>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<StillsListResponse>>, Error> {
    let user_id: String =
        query_scalar("SELECT user_id FROM users WHERE username = ?")
            .bind(&username)
            .fetch_one(&data.db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
                    Error::from_string("사용자를 찾을 수 없습니다.", StatusCode::NOT_FOUND)
                }
                _ => Error::from_string(
                    format!("조회 실패: {}", e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ),
            })?;

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let total: i64 = query_scalar("SELECT COUNT(*) FROM stills WHERE user_id = ?")
        .bind(&user_id)
        .fetch_one(&data.db)
        .await
        .unwrap_or(0);

    let stills: Vec<StillResponse> = query_as(
        r#"
        SELECT s.still_id, s.slug, s.user_id, u.username, u.display_name,
               s.caption, s.image_url, s.width, s.height, s.published_at
        FROM stills s JOIN users u ON s.user_id = u.user_id
        WHERE s.user_id = ?
        ORDER BY s.published_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(&user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        poem::Error::from_string(
            format!("stills 조회 실패: {}", e),
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    Ok(Json(CustomResponse::ok(StillsListResponse { stills, total })))
}
