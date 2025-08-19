pub mod auth;
pub mod config;
pub mod csrf;
pub mod debug_middleware;
pub mod manifest;
pub mod models;
pub mod repositories;

use axum::{
    Router,
    extract::State,
    response::Html,
    routing::{get, post},
};
use config::AppConfig;
use maud::{DOCTYPE, html};
use repositories::SessionRepository;
use sqlx::SqlitePool;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: AppConfig,
}

async fn index(State(state): State<AppState>, req: axum::extract::Request) -> Html<String> {
    // Try to get CSRF token from session
    let csrf_token = if let Some(cookie_header) = req.headers().get("Cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            // Parse cookies to find session_id
            let mut session_id: Option<&str> = None;
            for cookie_pair in cookie_str.split(';') {
                let cookie_pair = cookie_pair.trim();
                if let Some(id) = cookie_pair.strip_prefix("session_id=") {
                    session_id = Some(id);
                    break;
                }
            }

            if let Some(session_id) = session_id {
                let session_repo = SessionRepository::new(&state.pool);
                if let Ok(Some(session)) = session_repo.find_by_uuid(session_id).await {
                    Some(session.csrf_token)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let markup = html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                @if let Some(token) = csrf_token {
                    meta name="csrf-token" content=(token);
                }
                script defer src={ "/public/" (manifest::javascript_filename()) } {}
                title { "Kore Douyo" }
            }
            body {
                div id="root" {}
            }
        }
    };
    Html(markup.into_string())
}

async fn get_database_conn_pool(database_url: &str) -> SqlitePool {
    // Connect to database
    match SqlitePool::connect(database_url).await {
        Ok(pool) => {
            info!("Successfully connected to database at {}", database_url);
            pool
        }
        Err(e) => {
            panic!("Database connection failed: {}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "kore_douyo=debug,tower_http=debug,axum::rejection=trace,sqlx=debug".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::load_or_default();
    info!(
        "Loaded configuration: Server {}:{}, Database URL: {}",
        config.server.host, config.server.port, config.database.url
    );

    // Connect to database
    info!("Connecting to database...");
    let pool = get_database_conn_pool(&config.database.url).await;
    info!("Database connection established");

    // Create app state
    let state = AppState {
        pool,
        config: config.clone(),
    };

    // Create auth routes
    let auth_routes = Router::new()
        .route("/register", post(auth::handlers::register))
        .route("/login", post(auth::handlers::login))
        .route("/logout", post(auth::handlers::logout))
        .with_state(state.clone());

    // Create board routes
    let board_routes = Router::new()
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            csrf::middleware::csrf_protection_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::session_auth_middleware,
        ))
        .with_state(state.clone());

    // Create public routes (no authentication required)
    let public_routes = Router::new().with_state(state.clone());

    let app = Router::new()
        .route("/", get(index))
        .nest("/api/auth", auth_routes)
        .nest("/api/boards", board_routes)
        .nest("/api/shared", public_routes)
        .nest_service("/public", ServeDir::new("frontend/dist"))
        .fallback(index)
        .with_state(state.clone())
        .layer(axum::middleware::from_fn_with_state(
            config.clone(),
            debug_middleware::debug_sleep_middleware,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::extract::Request| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|_request: &axum::extract::Request, _span: &tracing::Span| {
                    tracing::info!("Started processing request")
                })
                .on_response(
                    |_response: &axum::response::Response,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(latency = ?latency, "Request completed")
                    },
                ),
        );

    let bind_address = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    info!(
        "Server running on http://{}:{}",
        config.server.host, config.server.port
    );

    axum::serve(listener, app).await.unwrap();
}
