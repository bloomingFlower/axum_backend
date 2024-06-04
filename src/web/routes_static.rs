use crate::config::load_config;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::routing::{any_service, MethodRouter};
use tower_http::services::ServeDir;

/// Serve the static files from the WEB_FOLDER
// MethodRouter is a router that matches the HTTP method of the request.
pub fn serve_dir() -> MethodRouter {
    // Handle the 404 error
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Resource not found!")
    }
    // Any service is a service that matches any request.
    any_service(
        // Generate the ServeDir service with the WEB_FOLDER and the handle_404 service
        ServeDir::new(&load_config().WEB_FOLDER.clone())
            .not_found_service(handle_404.into_service()),
    )
}
