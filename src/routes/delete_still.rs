use std::sync::Arc;

use inert_api::auth::authorization::current_user;
use poem::{handler, http::StatusCode, web::{Data, Json, Path}, Error, Request};
use sqlx::{query, query_scalar};

use crate::models::{AppState, CustomResponse};

#[derive(serde::Serialize)]
pub struct DeleteStillResponse {
    pub still_id: String,
}

#[handler]
pub async fn delete_still(
    req: &Request,
    Path(still_id): Path<String>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<DeleteStillResponse>>, Error> {
    let user = current_user(req)?;

    let owner_id: String = query_scalar("SELECT user_id FROM stills WHERE still_id = ?")
        .bind(&still_id)
        .fetch_one(&data.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                Error::from_string("still을 찾을 수 없습니다.", StatusCode::NOT_FOUND)
            }
            _ => Error::from_string(format!("조회 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR),
        })?;

    if owner_id != user.user_id {
        return Err(Error::from_string("본인의 still만 삭제할 수 있습니다.", StatusCode::FORBIDDEN));
    }

    query("DELETE FROM stills WHERE still_id = ?")
        .bind(&still_id)
        .execute(&data.db)
        .await
        .map_err(|e| {
            Error::from_string(format!("삭제 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    Ok(Json(CustomResponse::ok(DeleteStillResponse { still_id })))
}
