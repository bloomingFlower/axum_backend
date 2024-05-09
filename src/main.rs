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

mod error;
mod model;
mod web;
//mod model;

#[tokio::main]
async fn main() -> Result<()> {
    let mc = ModelController::new().await?;
    let routes_apis = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static());

    // region: Start the server
    let listener = TcpListener::bind("127.0.0.1:3000").await.or_else(|err| {
        eprintln!("Could not bind to port 3000: {} (run lsof -i :3000)", err);
        std::process::exit(1);
    })?;
    println!("--> Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();
    // endregion: End the server

    Ok(())
}

async fn main_response_mapper(res: Response) -> Response {
    println!("--> {:<12} - main_response_mapper", "RES_MAPPER");
    println!();
    res
}

fn routes_static() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

//region Hello Routes
fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/:name", get(handler_hello2))
}
//endregion

//region: Handler Hello
#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

// e.g., `GET /hello?name=foo`
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("--> {:<12} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World");
    Html(format!("Hello, <strong>{name}</strong>"))
}

// e.g., `GET /hello2/foo`
async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("--> {:<12} - handler_hello2 - {name:?}", "HANDLER");

    Html(format!("Hello, <strong>{name}</strong>"))
}
//endregion: Handler Hello
