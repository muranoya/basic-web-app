use crate::AppState;
use crate::auth::errors::{AuthError, AuthResult};
use crate::models::Session;
use crate::repositories::{SessionRepository, UserRepository};
use axum::response::IntoResponse;
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header::SET_COOKIE},
};
use bcrypt::{DEFAULT_COST, hash, verify};
use serde::Deserialize;
use tracing::{debug, info, instrument};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[instrument(skip(state, request), fields(email = %request.email), )]
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> AuthResult<impl IntoResponse> {
    let user_repo = UserRepository::new(&state.pool);
    let session_repo = SessionRepository::new(&state.pool);

    // Check if user already exists
    if user_repo.exists_by_email(&request.email).await? {
        debug!("Registration failed: email already exists");
        return Err(AuthError::EmailAlreadyExists);
    }

    let password_hash = hash(&request.password, DEFAULT_COST)?;

    //let user_id = match user_repo.create(&request.email, &password_hash).await {
    //    Ok(user_id) => user_id,
    //    Err(e) => {
    //        error!("{}", e);
    //        return Err(AuthError::InternalError);
    //    }
    //};
    let user_id = user_repo.create(&request.email, &password_hash).await?;
    info!(user_id = %user_id, "User created successfully");

    // Create session
    let session = Session::new(
        user_id, None, // TODO: Extract device info from headers
        None, // TODO: Extract IP address from request
    );
    let session_uuid = session.get_session_uuid().to_string();
    session_repo.create(&session).await?;

    // Set cookie with session UUID (30 days = 2592000 seconds)
    let cookie = format!(
        "session_id={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=2592000",
        session_uuid
    );

    let mut res = (StatusCode::OK).into_response();
    res.headers_mut()
        .insert(SET_COOKIE, cookie.parse().unwrap());
    Ok(res)
}

#[instrument(skip(state, request), fields(email = %request.email))]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> AuthResult<impl IntoResponse> {
    let user_repo = UserRepository::new(&state.pool);
    let session_repo = SessionRepository::new(&state.pool);

    // Find user by email
    let user = user_repo
        .find_by_email(&request.email)
        .await?
        .ok_or_else(|| {
            debug!("Login failed: user not found");
            AuthError::InvalidCredentials
        })?;

    // Verify password
    if !verify(&request.password, &user.password)? {
        debug!("Login failed: invalid password");
        return Err(AuthError::InvalidCredentials);
    }

    // Create session
    let session = Session::new(
        user.id, None, // TODO: Extract device info from headers
        None, // TODO: Extract IP address from request
    );
    let session_uuid = session.get_session_uuid().to_string();
    session_repo.create(&session).await?;

    // Set cookie with session UUID (30 days = 2592000 seconds)
    let cookie = format!(
        "session_id={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=2592000",
        session_uuid
    );

    let mut res = (StatusCode::OK).into_response();
    res.headers_mut()
        .insert(SET_COOKIE, cookie.parse().unwrap());
    Ok(res)
}

#[instrument(skip(state))]
pub async fn logout(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> AuthResult<impl IntoResponse> {
    let session_repo = SessionRepository::new(&state.pool);

    // Extract session ID from cookie
    if let Some(cookie_header) = req.headers().get("Cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            // Parse cookies to find session_id
            for cookie_pair in cookie_str.split(';') {
                let cookie_pair = cookie_pair.trim();
                if let Some(session_id) = cookie_pair.strip_prefix("session_id=") {
                    // Delete session from database
                    session_repo.delete_by_uuid(session_id).await?;
                    break;
                }
            }
        }
    }

    // Clear cookie by setting it to expire immediately
    let clear_cookie = "session_id=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0";

    let mut res = (StatusCode::OK).into_response();
    res.headers_mut()
        .insert(SET_COOKIE, clear_cookie.parse().unwrap());

    Ok(res)
}
