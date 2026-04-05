use std::sync::Arc;

use inert_api::auth::authorization::current_user;
use poem::{handler, web::{Data, Json, Query}, Error, Request};
use sqlx::{query_as, query_scalar};

use crate::models::{AppState, CustomResponse, PaginationQuery, StillRow, StillsListResponse};

#[handler]
pub async fn get_mine(
    req: &Request,
    Query(params): Query<PaginationQuery>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<StillsListResponse>>, Error> {
    let user = current_user(req)?;
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let total: i64 = query_scalar("SELECT COUNT(*) FROM stills WHERE user_id = ?")
        .bind(&user.user_id)
        .fetch_one(&data.db)
        .await
        .unwrap_or(0);

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
        WHERE s.user_id = ?
        GROUP BY s.still_id
        ORDER BY s.published_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(&user.user_id)
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

    let stills = rows.into_iter().map(StillRow::into_response).collect();
    Ok(Json(CustomResponse::ok(StillsListResponse { stills, total })))
}
