// crates/web/src/extractors/mod.rs

use crate::error_response::AppError;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use owasp_core::error::ApiError;
use owasp_core::models::User;

pub struct AuthenticatedUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(user) = parts.extensions.get::<User>() {
            Ok(AuthenticatedUser(user.clone()))
        } else {
            Err(ApiError::Unauthorized.into())
        }
    }
}

pub struct OptionalUser(pub Option<User>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(user) = parts.extensions.get::<User>() {
            Ok(OptionalUser(Some(user.clone())))
        } else {
            Ok(OptionalUser(None))
        }
    }
}
