use crate::{Error, Result};
use axum::routing::post;
use axum::Router;
use axum::Json;
use serde_json::{json, Value};
use serde::Deserialize;


pub fn routes() -> Router {
    Router::new()
        .route("/api/login", post(api_login))
}

async fn api_login(payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("--> {:<12} - api_login - {payload:?}", "HANDLER", payload = payload);

    if payload.username != "demo1" || payload.password != "demo" {
        return Err(Error::LoginFail)
    }

    // Set Cookie

    let body = Json(json!({
        "result": {
            "status": "ok"
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}