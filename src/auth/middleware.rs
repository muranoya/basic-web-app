use crate::AppState;
use crate::auth::errors::AuthError;
use crate::models::User;
use crate::repositories::{SessionRepository, UserRepository};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user: User,
}

pub async fn session_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let cookie_header = request
        .headers()
        .get("Cookie")
        .and_then(|header| header.to_str().ok())
        .ok_or(AuthError::MissingCookieHeader)?;

    // Parse cookies to find session_id
    let mut session_id: Option<&str> = None;
    for cookie_pair in cookie_header.split(';') {
        let cookie_pair = cookie_pair.trim();
        if let Some(id) = cookie_pair.strip_prefix("session_id=") {
            session_id = Some(id);
            break;
        }
    }

    let session_id = session_id.ok_or(AuthError::MissingSessionId)?;
    let session_repo = SessionRepository::new(&state.pool);
    let user_repo = UserRepository::new(&state.pool);

    // Find session in database
    let session = session_repo
        .find_by_uuid(session_id)
        .await
        .map_err(|_| AuthError::NotLogined)?
        .ok_or(AuthError::NotLogined)?;

    // Get user information
    let user = user_repo
        .get_by_id(session.user_id)
        .await
        .map_err(|_| AuthError::UserNotFound)?;

    // Create AuthenticatedUser
    let authenticated_user = AuthenticatedUser { user: user };

    // Add authenticated user to request extensions
    request.extensions_mut().insert(authenticated_user);

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}
