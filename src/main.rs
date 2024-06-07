/// Import the necessary modules
mod config;
mod ctx;
mod error;
mod log;
mod model;
mod pwd;
mod rpc;
mod token;
mod utils;
mod web;
// #[cfg(test)] // Commented during the early development
mod _dev_utils;

/// Export the Error and Result types
pub use self::error::{Error, Result};

/// Import the necessary modules
use crate::model::ModelManager;
use crate::web::mw_auth::{mw_ctx_resolver, mw_require_auth};
use crate::web::mw_res_map::main_response_mapper;
use crate::web::{routes_login, routes_rpc, routes_static};

use axum::{middleware, Router};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Tokio Runtime Entry Point
#[tokio::main]
/// Async Main Function that returns a Result
async fn main() -> Result<()> {
    tracing_subscriber::fmt() // For standard output
        .without_time() // For local development(simplicity)
        .with_target(false) // For local development(simplicity)
        .with_env_filter(EnvFilter::from_default_env()) // set log level(config.toml)
        .init();

    // For DEV ONLY
    _dev_utils::init_dev().await;

    // Initialize the Model Manager and wait for it to be ready
    let mm = ModelManager::new().await?;

    // Initialize the new Router with the Hello Routes
    // let routes_hello = Router::new()
    //     // Http and DB operations should be async
    //     .route("/hello", get(|| async { Html("Hello World") }))
    //     // Check ctx and token
    //     .route_layer(middleware::from_fn(mw_require_auth));

    let routes_rpc =
        routes_rpc::routes(mm.clone()).route_layer(middleware::from_fn(mw_require_auth));

    // Initialize the Router with all the routes
    let routes_all = Router::new()
        // Merge the Login Routes
        .merge(routes_login::routes(mm.clone()))
        // Merge the Hello Routes
        // .merge(routes_hello)
        // Nest the API Routes under the /api path
        .nest("/api", routes_rpc)
        // Add a middleware to map the all responses
        .layer(middleware::map_response(main_response_mapper))
        // Add a middleware to resolve the context
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolver))
        // Add a middleware to manage cookies
        .layer(CookieManagerLayer::new())
        // Add a fallback service to serve static files
        .fallback_service(routes_static::serve_dir());

    // region: Start the server
    // Create a TCP Listener on port 3000
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    // Start the server and handle errors gracefully
    if let Err(err) = axum::serve(listener, routes_all.into_make_service()).await {
        eprintln!("Server error: {}", err);
    }

    // endregion: End the server
    // Return Ok if everything went well
    Ok(())
}

// //region Hello Routes
// /// Hello Routes under the root path
// fn routes_hello() -> Router {
//     Router::new()
//         // e.g., `GET /hello` with a query parameter
//         .route("/hello", get(handler_hello))
//         // e.g., `GET /hello2/foo` with a path parameter
//         .route("/hello2/:name", get(handler_hello2))
// }
// //endregion
//
// //region: Handler Hello
// /// Hello Parameters
// // Debug is used to print the struct({params:?}) in the log
// // serde Deserialize is used to parse the query parameters(JSON, Query String, etc.)
// #[derive(Debug, Deserialize)]
// struct HelloParams {
//     // Option is used to make the field optional
//     name: Option<String>,
// }
//
// /// Hello Handler with a query parameter
// // e.g., `GET /hello?name=foo`
// // The query parameter is extracted from the request
// async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
//     debug!(" {:<12} - handler_hello - {params:?}", "HANDLER");
//     // Unwrap the name parameter or use the default value "World"
//     let name = params.name.as_deref().unwrap_or("World");
//     // Return an HTML response with the name
//     Html(format!("Hello, <strong>{name}</strong>"))
// }
//
// // e.g., `GET /hello2/foo`
// // The path parameter is extracted from the request
// async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
//     debug!(" {:<12} - handler_hello2 - {name:?}", "HANDLER");
//
//     Html(format!("Hello, <strong>{name}</strong>"))
// }
// //endregion: Handler Hello
