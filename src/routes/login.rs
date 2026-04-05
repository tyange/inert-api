use std::{env, sync::Arc};

use inert_api::auth::jwt::Claims;
use poem::{handler, http::StatusCode, web::{Data, Json}, Error};
use sqlx::query_as;

use crate::models::{AppState, CustomResponse, LoginRequest, LoginResponse};

#[derive(sqlx::FromRow)]
struct UserRow {
    user_id: String,
    password: String,
}

#[handler]
pub async fn login(
    Json(body): Json<LoginRequest>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<LoginResponse>>, Error> {
    let user: UserRow = query_as("SELECT user_id, password FROM users WHERE email = ?")
        .bind(&body.email)
        .fetch_one(&data.db)
        .await
        .map_err(|_| Error::from_string("이메일 또는 비밀번호가 올바르지 않습니다.", StatusCode::UNAUTHORIZED))?;

    let valid = bcrypt::verify(&body.password, &user.password).map_err(|e| {
        Error::from_string(format!("비밀번호 검증 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    if !valid {
        return Err(Error::from_string("이메일 또는 비밀번호가 올바르지 않습니다.", StatusCode::UNAUTHORIZED));
    }

    let secret = env::var("JWT_SECRET")
        .map_err(|_| Error::from_string("서버 설정 오류", StatusCode::INTERNAL_SERVER_ERROR))?;

    let access_token = Claims::create_access_token(&user.user_id, secret.as_bytes())?;

    Ok(Json(CustomResponse::ok(LoginResponse { access_token })))
}
