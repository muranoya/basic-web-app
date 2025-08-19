use crate::config::AppConfig;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::time::Duration;
use tokio::time::sleep;

pub async fn debug_sleep_middleware(
    State(config): State<AppConfig>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(debug_config) = &config.debug {
        if debug_config.inject_sleep {
            sleep(Duration::from_millis(debug_config.sleep_millis)).await;
        }
    }

    next.run(request).await
}
