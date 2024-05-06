use crate::{web, Error, Result};
use axum::routing::post;
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login))
}

async fn api_login(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!(
        "--> {:<12} - api_login - {payload:?}",
        "HANDLER",
        payload = payload
    );

    // TODO: Implement a real database check
    if payload.username != "demo1" || payload.password != "demo" {
        return Err(Error::LoginFail);
    }

    // FIXME: Implement a real token generation
    cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));

    // Create the response body
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
