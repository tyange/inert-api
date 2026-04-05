use std::{env, sync::Arc};

use inert_api::auth::{google::GoogleTokenVerifier, jwt::Claims};
use poem::{handler, http::StatusCode, web::{Data, Json}, Error};
use sqlx::{query, query_scalar};
use uuid::Uuid;

use crate::models::{AppState, CustomResponse, GoogleLoginRequest, LoginResponse};

#[handler]
pub async fn login_google(
    Json(body): Json<GoogleLoginRequest>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<LoginResponse>>, Error> {
    let client_id = env::var("GOOGLE_CLIENT_ID").map_err(|_| {
        Error::from_string("서버 설정 오류", StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    let secret = env::var("JWT_SECRET").map_err(|_| {
        Error::from_string("서버 설정 오류", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let google_user = GoogleTokenVerifier::new()
        .verify_id_token(&body.id_token, &client_id)
        .await?;

    // google_sub로 기존 유저 조회
    let user_id: Option<String> = query_scalar(
        "SELECT user_id FROM users WHERE google_sub = ? OR email = ?",
    )
    .bind(&google_user.google_sub)
    .bind(&google_user.email)
    .fetch_optional(&data.db)
    .await
    .map_err(|e| Error::from_string(format!("DB 조회 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;

    let user_id = match user_id {
        Some(id) => {
            // google_sub, display_name, avatar_url 업데이트
            query(
                "UPDATE users SET google_sub = ?, display_name = COALESCE(NULLIF(TRIM(display_name),''), ?), avatar_url = COALESCE(NULLIF(TRIM(avatar_url),''), ?) WHERE user_id = ?",
            )
            .bind(&google_user.google_sub)
            .bind(&google_user.display_name)
            .bind(&google_user.avatar_url)
            .bind(&id)
            .execute(&data.db)
            .await
            .map_err(|e| Error::from_string(format!("DB 업데이트 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;
            id
        }
        None => {
            // 새 유저 생성
            let new_id = Uuid::new_v4().to_string();
            let username = google_user.email
                .split('@')
                .next()
                .unwrap_or("user")
                .to_string();
            // username 중복 시 uuid 앞 8자 붙이기
            let username = format!("{}_{}", username, &new_id[..8]);

            query(
                "INSERT INTO users (user_id, email, username, password, google_sub, display_name, avatar_url) VALUES (?, ?, ?, '', ?, ?, ?)",
            )
            .bind(&new_id)
            .bind(&google_user.email)
            .bind(&username)
            .bind(&google_user.google_sub)
            .bind(&google_user.display_name)
            .bind(&google_user.avatar_url)
            .execute(&data.db)
            .await
            .map_err(|e| Error::from_string(format!("유저 생성 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;
            new_id
        }
    };

    let access_token = Claims::create_access_token(&user_id, secret.as_bytes())?;
    Ok(Json(CustomResponse::ok(LoginResponse { access_token })))
}
