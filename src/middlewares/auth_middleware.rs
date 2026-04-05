use std::env;

use inert_api::auth::{authorization::AuthenticatedUser, jwt::Claims};
use poem::{http::StatusCode, Endpoint, Error, Middleware, Request};

fn authenticated_user_from_request(req: &Request) -> Result<AuthenticatedUser, Error> {
    let header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| Error::from_string("Authorization 헤더가 필요합니다.", StatusCode::UNAUTHORIZED))?
        .to_str()
        .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or_else(|| Error::from_string("Bearer 토큰 형식이 아닙니다.", StatusCode::UNAUTHORIZED))?;

    let secret = env::var("JWT_SECRET").map_err(|_| {
        Error::from_string("서버 설정 오류", StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    let secret_bytes = secret.as_bytes();

    let token_data = Claims::from_token(token, secret_bytes)?;
    let is_valid = Claims::validate_token(token, secret_bytes)?;

    if is_valid {
        Ok(AuthenticatedUser { user_id: token_data.claims.sub })
    } else {
        Err(Error::from_string("만료된 토큰입니다.", StatusCode::UNAUTHORIZED))
    }
}

pub struct Auth;

impl<E: Endpoint> Middleware<E> for Auth {
    type Output = AuthImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        AuthImpl { ep }
    }
}

pub struct AuthImpl<E> {
    ep: E,
}

impl<E: Endpoint> Endpoint for AuthImpl<E> {
    type Output = E::Output;

    async fn call(&self, mut req: Request) -> Result<Self::Output, Error> {
        let user = authenticated_user_from_request(&req)?;
        req.extensions_mut().insert(user);
        self.ep.call(req).await
    }
}
