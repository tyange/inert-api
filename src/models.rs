use aws_sdk_s3::Client as S3Client;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

pub struct AppState {
    pub db: Pool<Sqlite>,
    pub s3: S3Client,
    pub bucket_name: String,
    pub cdn_base_url: String,
}

// ─── 공통 응답 래퍼 ─────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CustomResponse<T: Serialize> {
    pub status: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> CustomResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { status: true, data: Some(data), message: None }
    }

    pub fn ok_msg(data: T, msg: &str) -> Self {
        Self { status: true, data: Some(data), message: Some(msg.to_owned()) }
    }
}

// ─── Auth ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleLoginRequest {
    pub id_token: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MeResponse {
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

// ─── Stills ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateStillRequest {
    pub image_keys: Vec<String>,
    pub caption: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ImageResponse {
    pub image_id: String,
    pub image_url: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub position: i64,
}

/// DB에서 직접 매핑되는 내부 row 타입 (images_json 포함)
#[derive(Debug, FromRow)]
pub struct StillRow {
    pub still_id: String,
    pub slug: String,
    pub user_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub caption: Option<String>,
    pub images_json: String,
    pub published_at: String,
}

impl StillRow {
    pub fn into_response(self) -> StillResponse {
        let images: Vec<ImageResponse> =
            serde_json::from_str(&self.images_json).unwrap_or_default();
        StillResponse {
            still_id: self.still_id,
            slug: self.slug,
            user_id: self.user_id,
            username: self.username,
            display_name: self.display_name,
            caption: self.caption,
            images,
            published_at: self.published_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StillResponse {
    pub still_id: String,
    pub slug: String,
    pub user_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub caption: Option<String>,
    pub images: Vec<ImageResponse>,
    pub published_at: String,
}

#[derive(Debug, Serialize)]
pub struct StillsListResponse {
    pub stills: Vec<StillResponse>,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ─── Images ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UploadImageResponse {
    pub image_key: String,
    pub image_url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
