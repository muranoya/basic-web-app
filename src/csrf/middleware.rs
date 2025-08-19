use crate::AppState;
use crate::repositories::SessionRepository;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn csrf_protection_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    method: Method,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip CSRF check for GET requests
    if method == Method::GET {
        return Ok(next.run(request).await);
    }

    // Get CSRF token from headers
    let csrf_token = headers
        .get("X-CSRF-Token")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Get session cookie
    let cookie_header = headers
        .get("Cookie")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let mut session_id: Option<&str> = None;
    for cookie_pair in cookie_header.split(';') {
        let cookie_pair = cookie_pair.trim();
        if let Some(id) = cookie_pair.strip_prefix("session_id=") {
            session_id = Some(id);
            break;
        }
    }

    let session_id = session_id.ok_or(StatusCode::BAD_REQUEST)?;
    let session_repo = SessionRepository::new(&state.pool);

    // Verify CSRF token matches session
    let session = session_repo
        .find_by_uuid(session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if session.csrf_token != csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
