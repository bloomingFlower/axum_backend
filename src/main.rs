pub use self::error::{Error, Result};

use axum::extract::{Path, Query};
use axum::Router;
use axum::routing::{get, get_service};
use axum::response::{Html, IntoResponse};
use tokio::net::TcpListener;
use serde::Deserialize;
use tower_http::services::ServeDir;

mod error;
mod web;

#[tokio::main]
async fn main() {
    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .fallback_service(routes_static());


    // region: Start the server
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("--> Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, routes_all.into_make_service()).await.unwrap();
    // endregion: End the server

}


fn routes_static() -> Router {
    Router::new()
        .nest_service("/", get_service(ServeDir::new("./")))
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
struct  HelloParams {
    name: Option<String>
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