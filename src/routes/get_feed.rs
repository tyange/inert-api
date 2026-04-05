use std::sync::Arc;

use poem::{handler, web::{Data, Json, Query}, Error};
use sqlx::{query_as, query_scalar};

use crate::models::{AppState, CustomResponse, PaginationQuery, StillResponse, StillsListResponse};

#[handler]
pub async fn get_feed(
    Query(params): Query<PaginationQuery>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<StillsListResponse>>, Error> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let total: i64 = query_scalar("SELECT COUNT(*) FROM stills")
        .fetch_one(&data.db)
        .await
        .unwrap_or(0);

    let stills: Vec<StillResponse> = query_as(
        r#"
        SELECT s.still_id, s.slug, s.user_id, u.username, u.display_name,
               s.caption, s.image_url, s.width, s.height, s.published_at
        FROM stills s JOIN users u ON s.user_id = u.user_id
        ORDER BY s.published_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        poem::Error::from_string(
            format!("피드 조회 실패: {}", e),
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    Ok(Json(CustomResponse::ok(StillsListResponse { stills, total })))
}
