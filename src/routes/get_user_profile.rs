use std::sync::Arc;

use poem::{handler, http::StatusCode, web::{Data, Json, Path}, Error};
use sqlx::query_as;

use crate::models::{AppState, CustomResponse, UserProfileResponse};

#[handler]
pub async fn get_user_profile(
    Path(username): Path<String>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<UserProfileResponse>>, Error> {
    let profile: UserProfileResponse = query_as(
        "SELECT user_id, username, display_name, avatar_url, bio FROM users WHERE username = ?",
    )
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

    Ok(Json(CustomResponse::ok(profile)))
}
