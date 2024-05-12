pub use self::error::{Error, Result};

use crate::model::ModelController;
use axum::extract::{Path, Query};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, get_service};
use axum::{middleware, Router};
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

mod ctx;
mod error;
mod model;
mod web;

/// Tokio Runtime Entry Point
#[tokio::main]
/// Async Main Function that returns a Result
async fn main() -> Result<()> {
    // Initialize the Model Controller and wait for it to be ready
    let mc = ModelController::new().await?;
    // Initialize the Router with the Model Controller
    let routes_apis = web::routes_tickets::routes(mc.clone())
        // Add a middleware to require authentication
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));
    // Initialize the Router with all the routes
    let routes_all = Router::new()
        // Merge the Hello Routes
        .merge(routes_hello())
        // Merge the Login Routes
        .merge(web::routes_login::routes())
        // Nest the API Routes under the /api path
        .nest("/api", routes_apis)
        // Add a middleware to map the all responses
        .layer(middleware::map_response(main_response_mapper))
        // Add a middleware to manage cookies
        .layer(CookieManagerLayer::new())
        // Add a fallback service to serve static files
        .fallback_service(routes_static());

    // region: Start the server
    // Create a TCP Listener on port 3000
    let listener = TcpListener::bind("127.0.0.1:3000").await.or_else(|err| {
        eprintln!("Could not bind to port 3000: {} (run lsof -i :3000)", err);
        // Exit the process with an error code if the listener could not be created
        std::process::exit(1);
    })?;
    println!("--> Listening on {}", listener.local_addr().unwrap());
    // Start the server and handle errors gracefully
    if let Err(err) = axum::serve(listener, routes_all.into_make_service()).await {
        eprintln!("Server error: {}", err);
    }

    // endregion: End the server
    // Return Ok if everything went well
    Ok(())
}

/// Main Response Mapper Middleware
async fn main_response_mapper(res: Response) -> Response {
    println!("--> {:<12} - main_response_mapper", "RES_MAPPER");
    println!();
    res
}

/// Static Routes under the root path
fn routes_static() -> Router {
    // Serve the current directory as static files under the root path
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

//region Hello Routes
/// Hello Routes under the root path
fn routes_hello() -> Router {
    Router::new()
        // e.g., `GET /hello` with a query parameter
        .route("/hello", get(handler_hello))
        // e.g., `GET /hello2/foo` with a path parameter
        .route("/hello2/:name", get(handler_hello2))
}
//endregion

//region: Handler Hello
/// Hello Parameters
// Debug is used to print the struct({params:?}) in the log
// serde Deserialize is used to parse the query parameters(JSON, Query String, etc.)
#[derive(Debug, Deserialize)]
struct HelloParams {
    // Option is used to make the field optional
    name: Option<String>,
}

/// Hello Handler with a query parameter
// e.g., `GET /hello?name=foo`
// The query parameter is extracted from the request
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("--> {:<12} - handler_hello - {params:?}", "HANDLER");
    // Unwrap the name parameter or use the default value "World"
    let name = params.name.as_deref().unwrap_or("World");
    // Return an HTML response with the name
    Html(format!("Hello, <strong>{name}</strong>"))
}

// e.g., `GET /hello2/foo`
// The path parameter is extracted from the request
async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("--> {:<12} - handler_hello2 - {name:?}", "HANDLER");

    Html(format!("Hello, <strong>{name}</strong>"))
}
//endregion: Handler Hello
