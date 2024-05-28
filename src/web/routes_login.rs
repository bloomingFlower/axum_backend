use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForLogin};
use crate::model::ModelManager;
use crate::web::{self, Error, Result};
use axum::extract::State;
use axum::routing::post;
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

/// Create the Login Routes and return the Router
pub fn routes(mm: ModelManager) -> Router {
    // Create the Login Route with the POST method and the api_login handler
    Router::new()
        .route("/api/login", post(api_login_handler))
        .with_state(mm)
}

/// Login Handler that returns a JSON response with a status
async fn api_login_handler(
    State(mm): State<ModelManager>,
    cookies: Cookies,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_login_handler", "HANDLER");

    let LoginPayload {
        username,
        password: pwd_clear,
    } = payload;
    let root_ctx = Ctx::root_ctx();

    let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?;

    let user_id = user.id;
    let Some(pwd) = user.password else {
        return Err(Error::LoginFailUserHasNoPwd { user_id });
    };

    pwd::validate_pwd(
        &EncryptContent {
            salt: user.password_salt.to_string(),
            content: pwd_clear.clone(),
        },
        &pwd,
    )
    .map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

    // FIXME: Implement a real token generation
    // Add a cookie with the token to the response cookies list
    cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));

    // Create the response body
    let body = Json(json!({
        "result": {
            "success": true
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
