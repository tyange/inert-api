use poem::{http::StatusCode, Error, Request};

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: String,
}

pub fn current_user(req: &Request) -> Result<&AuthenticatedUser, Error> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| Error::from_string("인증된 사용자 정보가 없습니다.", StatusCode::UNAUTHORIZED))
}
