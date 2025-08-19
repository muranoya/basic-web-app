use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Not Logined")]
    NotLogined,

    #[error("Missing Cookie header")]
    MissingCookieHeader,

    #[error("Missing SessionId header")]
    MissingSessionId,

    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Password hashing error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),

    #[error("Repository error: {0}")]
    RepositoryError(#[from] anyhow::Error),

    #[error("Internal server error")]
    InternalError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::EmailAlreadyExists => (StatusCode::CONFLICT, "Email already exists"),
            AuthError::NotLogined => (StatusCode::UNAUTHORIZED, "Not Logined"),
            AuthError::MissingCookieHeader => {
                (StatusCode::UNAUTHORIZED, "Cookie header not found.")
            }
            AuthError::MissingSessionId => (StatusCode::UNAUTHORIZED, "Invalid session id"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AuthError::DatabaseError(_)
            | AuthError::BcryptError(_)
            | AuthError::RepositoryError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AuthError::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

pub type AuthResult<T> = Result<T, AuthError>;
