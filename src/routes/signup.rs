use std::sync::Arc;

use poem::{handler, http::StatusCode, web::{Data, Json}, Error};
use sqlx::query;
use uuid::Uuid;

use crate::models::{AppState, CustomResponse, SignupRequest};

#[derive(serde::Serialize)]
pub struct SignupResponse {
    pub user_id: String,
}

#[handler]
pub async fn signup(
    Json(body): Json<SignupRequest>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<SignupResponse>>, Error> {
    if body.email.is_empty() || body.username.is_empty() || body.password.is_empty() {
        return Err(Error::from_string("이메일, 사용자명, 비밀번호를 모두 입력해주세요.", StatusCode::BAD_REQUEST));
    }

    let password_hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST).map_err(|e| {
        Error::from_string(format!("비밀번호 처리 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let user_id = Uuid::new_v4().to_string();

    query("INSERT INTO users (user_id, email, username, password) VALUES (?, ?, ?, ?)")
        .bind(&user_id)
        .bind(&body.email)
        .bind(&body.username)
        .bind(&password_hash)
        .execute(&data.db)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                Error::from_string("이미 사용 중인 이메일 또는 사용자명입니다.", StatusCode::CONFLICT)
            } else {
                Error::from_string(format!("회원가입 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
            }
        })?;

    Ok(Json(CustomResponse::ok_msg(SignupResponse { user_id }, "회원가입에 성공했습니다.")))
}
