use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use poem::{http::StatusCode, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
}

impl Claims {
    pub fn new(user_id: &str, token_type: &str, expires_in_minutes: i64) -> Self {
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(expires_in_minutes))
            .expect("유효한 타임스탬프를 생성할 수 없습니다.")
            .timestamp() as usize;
        let iat = Utc::now().timestamp() as usize;
        Self {
            sub: user_id.to_owned(),
            exp: expiration,
            iat,
            token_type: token_type.to_owned(),
        }
    }

    pub fn to_token(&self, secret: &[u8]) -> Result<String, Error> {
        encode(&Header::default(), &self, &EncodingKey::from_secret(secret)).map_err(|e| {
            Error::from_string(
                format!("토큰 생성 실패: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })
    }

    pub fn from_token(token: &str, secret: &[u8]) -> Result<TokenData<Claims>, Error> {
        decode::<Claims>(token, &DecodingKey::from_secret(secret), &Validation::default())
            .map_err(|e| {
                Error::from_string(format!("토큰 파싱 실패: {}", e), StatusCode::UNAUTHORIZED)
            })
    }

    pub fn validate_token(token: &str, secret: &[u8]) -> Result<bool, Error> {
        match Self::from_token(token, secret) {
            Ok(token_data) => {
                let now = Utc::now().timestamp() as usize;
                Ok(now < token_data.claims.exp)
            }
            Err(e) => Err(Error::from_string(e.to_string(), StatusCode::UNAUTHORIZED)),
        }
    }

    pub fn create_access_token(user_id: &str, secret: &[u8]) -> Result<String, Error> {
        Self::new(user_id, "access", 60 * 24 * 7).to_token(secret)
    }
}
