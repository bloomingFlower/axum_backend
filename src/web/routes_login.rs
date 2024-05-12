use crate::{web, Error, Result};
use axum::routing::post;
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

/// Create the Login Routes and return the Router
pub fn routes() -> Router {
    // Create the Login Route with the POST method and the api_login handler
    Router::new().route("/api/login", post(api_login))
}

/// Login Handler that returns a JSON response with a status
async fn api_login(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!(
        "--> {:<12} - api_login - {payload:?}",
        "HANDLER",
        payload = payload
    );

    // TODO: Implement a real database check
    // Check the username and password
    if payload.username != "demo1" || payload.password != "demo" {
        // Return an error if the login fails
        return Err(Error::LoginFail);
    }

    // FIXME: Implement a real token generation
    // Add a cookie with the token to the response cookies list
    cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));

    // Create the response body
    let body = Json(json!({
        "result": {
            "status": "ok"
        }
    }));

    Ok(body)
}

/// Login Payload Struct for Deserialization
#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}
