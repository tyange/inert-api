use poem::{http::StatusCode, Error};
use reqwest::Client;
use serde::Deserialize;
use std::env;

#[derive(Clone)]
pub struct GoogleTokenVerifier {
    http_client: Client,
    tokeninfo_url: String,
}

#[derive(Debug, Clone)]
pub struct VerifiedGoogleUser {
    pub email: String,
    pub google_sub: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenInfoResponse {
    aud: Option<String>,
    iss: Option<String>,
    sub: Option<String>,
    email: Option<String>,
    name: Option<String>,
    picture: Option<String>,
    #[serde(default, deserialize_with = "deserialize_google_bool")]
    email_verified: bool,
    exp: Option<String>,
}

impl GoogleTokenVerifier {
    pub fn new() -> Self {
        let tokeninfo_url = env::var("GOOGLE_TOKENINFO_URL")
            .unwrap_or_else(|_| "https://oauth2.googleapis.com/tokeninfo".to_string());
        Self { http_client: Client::new(), tokeninfo_url }
    }

    pub async fn verify_id_token(
        &self,
        id_token: &str,
        client_id: &str,
    ) -> Result<VerifiedGoogleUser, Error> {
        let response = self
            .http_client
            .get(&self.tokeninfo_url)
            .query(&[("id_token", id_token)])
            .send()
            .await
            .map_err(|e| {
                eprintln!("Google 토큰 검증 요청 실패: {}", e);
                Error::from_string("Google 토큰 검증 실패", StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        if !response.status().is_success() {
            return Err(Error::from_string("유효하지 않은 Google 토큰", StatusCode::UNAUTHORIZED));
        }

        let token_info = response.json::<GoogleTokenInfoResponse>().await.map_err(|e| {
            eprintln!("Google 토큰 파싱 실패: {}", e);
            Error::from_string("Google 토큰 파싱 실패", StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        validate_token_info(&token_info, client_id)
    }
}

fn validate_token_info(
    info: &GoogleTokenInfoResponse,
    client_id: &str,
) -> Result<VerifiedGoogleUser, Error> {
    let aud = info.aud.as_deref().unwrap_or_default();
    if aud != client_id {
        return Err(Error::from_string("Google 토큰 audience 불일치", StatusCode::UNAUTHORIZED));
    }

    let iss = info.iss.as_deref().unwrap_or_default();
    if iss != "accounts.google.com" && iss != "https://accounts.google.com" {
        return Err(Error::from_string("Google 토큰 issuer 불일치", StatusCode::UNAUTHORIZED));
    }

    if !info.email_verified {
        return Err(Error::from_string("이메일 미인증 Google 계정", StatusCode::UNAUTHORIZED));
    }

    if let Some(exp) = info.exp.as_deref() {
        let expires_at = exp.parse::<i64>().unwrap_or(0);
        if chrono::Utc::now().timestamp() >= expires_at {
            return Err(Error::from_string("만료된 Google 토큰", StatusCode::UNAUTHORIZED));
        }
    }

    let email = info
        .email
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| Error::from_string("Google 이메일 없음", StatusCode::UNAUTHORIZED))?;

    let google_sub = info
        .sub
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| Error::from_string("Google sub 없음", StatusCode::UNAUTHORIZED))?;

    Ok(VerifiedGoogleUser {
        email: email.to_ascii_lowercase(),
        google_sub: google_sub.to_string(),
        display_name: info.name.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(str::to_string),
        avatar_url: info.picture.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(str::to_string),
    })
}

fn deserialize_google_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolValue {
        Bool(bool),
        Str(String),
    }
    let value = Option::<BoolValue>::deserialize(deserializer)?;
    Ok(match value {
        Some(BoolValue::Bool(b)) => b,
        Some(BoolValue::Str(s)) => s.eq_ignore_ascii_case("true"),
        None => false,
    })
}
