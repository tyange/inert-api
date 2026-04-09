use std::sync::Arc;

use aws_sdk_s3::primitives::ByteStream;
use inert_api::auth::authorization::current_user;
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json, Multipart},
    Error, IntoResponse, Request, Response,
};
use uuid::Uuid;

use crate::models::{AppState, CustomResponse, UploadImageResponse};

const MAX_BYTES: usize = 20 * 1024 * 1024; // 20MB

fn json_error(msg: &str, status: StatusCode) -> Error {
    let body = serde_json::to_string(&CustomResponse::<()>::error(msg)).unwrap();
    let resp = Response::builder()
        .status(status)
        .content_type("application/json; charset=utf-8")
        .body(body);
    Error::from_response(resp.into_response())
}

#[handler]
pub async fn upload_image(
    req: &Request,
    mut multipart: Multipart,
    data: Data<&Arc<AppState>>,
) -> Result<Json<CustomResponse<UploadImageResponse>>, Error> {
    let _user = current_user(req)?;

    while let Some(field) = multipart.next_field().await? {
        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_default();

        if !content_type.starts_with("image/") {
            return Err(json_error("이미지 파일만 업로드할 수 있습니다.", StatusCode::BAD_REQUEST));
        }

        let file_bytes = field
            .bytes()
            .await
            .map_err(|e| json_error(&e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

        if file_bytes.is_empty() {
            return Err(json_error("빈 파일은 업로드할 수 없습니다.", StatusCode::BAD_REQUEST));
        }

        if file_bytes.len() > MAX_BYTES {
            return Err(json_error("파일 크기가 20MB를 초과합니다.", StatusCode::PAYLOAD_TOO_LARGE));
        }

        // 이미지 검증 및 크기 추출
        let (width, height) = match image::load_from_memory(&file_bytes) {
            Ok(img) => (Some(img.width()), Some(img.height())),
            Err(_) => {
                return Err(json_error("유효하지 않은 이미지 파일입니다.", StatusCode::BAD_REQUEST));
            }
        };

        let extension = match content_type.as_str() {
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => "jpg",
        };

        let image_key = format!("stills/{}.{}", Uuid::new_v4(), extension);

        data.s3
            .put_object()
            .bucket(&data.bucket_name)
            .key(&image_key)
            .body(ByteStream::from(file_bytes))
            .content_type(&content_type)
            .send()
            .await
            .map_err(|e| {
                let detail = format!("S3 업로드 실패: {:?}", e);
                eprintln!("{}", detail);
                json_error(&detail, StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        let image_url = format!("{}/{}", data.cdn_base_url.trim_end_matches('/'), image_key);

        return Ok(Json(CustomResponse::ok(UploadImageResponse {
            image_key,
            image_url,
            width,
            height,
        })));
    }

    Err(json_error("업로드할 이미지가 없습니다.", StatusCode::BAD_REQUEST))
}
