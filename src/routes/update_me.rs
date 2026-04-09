use std::sync::Arc;

use inert_api::auth::authorization::current_user;
use poem::{handler, http::StatusCode, web::{Data, Json}, Error, Request};
use sqlx::{query, query_as};

use crate::models::{AppState, CustomResponse, MeResponse, UpdateMeRequest};

#[handler]
pub async fn update_me(
    req: &Request,
    Json(body): Json<UpdateMeRequest>,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<MeResponse>>, Error> {
    let user = current_user(req)?;

    if let Some(ref username) = body.username {
        let trimmed = username.trim();

        if trimmed.is_empty() || trimmed.len() > 30 {
            return Err(Error::from_string(
                "사용자명은 1~30자 이내여야 합니다.",
                StatusCode::BAD_REQUEST,
            ));
        }

        if !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(Error::from_string(
                "사용자명은 영문, 숫자, _, - 만 사용할 수 있습니다.",
                StatusCode::BAD_REQUEST,
            ));
        }

        // 중복 체크 (본인 제외)
        let exists: Option<(String,)> = query_as(
            "SELECT user_id FROM users WHERE username = ? AND user_id != ?",
        )
        .bind(trimmed)
        .bind(&user.user_id)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| Error::from_string(format!("DB 오류: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;

        if exists.is_some() {
            return Err(Error::from_string(
                "이미 사용 중인 사용자명입니다.",
                StatusCode::CONFLICT,
            ));
        }

        query("UPDATE users SET username = ? WHERE user_id = ?")
            .bind(trimmed)
            .bind(&user.user_id)
            .execute(&data.db)
            .await
            .map_err(|e| Error::from_string(format!("사용자명 변경 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;
    }

    if let Some(ref display_name) = body.display_name {
        let val = if display_name.trim().is_empty() { None } else { Some(display_name.trim()) };
        query("UPDATE users SET display_name = ? WHERE user_id = ?")
            .bind(val)
            .bind(&user.user_id)
            .execute(&data.db)
            .await
            .map_err(|e| Error::from_string(format!("프로필 변경 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;
    }

    if let Some(ref bio) = body.bio {
        let trimmed = bio.trim();
        if trimmed.chars().count() > 100 {
            return Err(Error::from_string(
                "소개는 100자 이내여야 합니다.",
                StatusCode::BAD_REQUEST,
            ));
        }
        let val = if trimmed.is_empty() { None } else { Some(trimmed) };
        query("UPDATE users SET bio = ? WHERE user_id = ?")
            .bind(val)
            .bind(&user.user_id)
            .execute(&data.db)
            .await
            .map_err(|e| Error::from_string(format!("프로필 변경 실패: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;
    }

    let me: MeResponse = query_as(
        "SELECT user_id, email, username, display_name, avatar_url, bio FROM users WHERE user_id = ?",
    )
    .bind(&user.user_id)
    .fetch_one(&data.db)
    .await
    .map_err(|_| Error::from_string("사용자를 찾을 수 없습니다.", StatusCode::NOT_FOUND))?;

    Ok(Json(CustomResponse::ok(me)))
}
