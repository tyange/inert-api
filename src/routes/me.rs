use std::sync::Arc;

use inert_api::auth::authorization::current_user;
use poem::{handler, http::StatusCode, web::{Data, Json}, Error, Request};
use sqlx::query_as;

use crate::models::{AppState, CustomResponse, MeResponse};

#[handler]
pub async fn me(
    req: &Request,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<MeResponse>>, Error> {
    let user = current_user(req)?;

    let me: MeResponse = query_as(
        "SELECT user_id, email, username, display_name, avatar_url, bio FROM users WHERE user_id = ?",
    )
    .bind(&user.user_id)
    .fetch_one(&data.db)
    .await
    .map_err(|_| Error::from_string("사용자를 찾을 수 없습니다.", StatusCode::NOT_FOUND))?;

    Ok(Json(CustomResponse::ok(me)))
}
