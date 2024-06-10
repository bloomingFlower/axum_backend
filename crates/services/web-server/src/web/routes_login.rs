use crate::web::{self, remove_token_cookie, Error, Result};
use axum::extract::State;
use axum::routing::post;
use axum::Json;
use axum::Router;
use lib_auth::pwd::{self, ContentToHash};
use lib_core::ctx::Ctx;
use lib_core::model::user::{UserBmc, UserForLogin};
use lib_core::model::ModelManager;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::Cookies;
use tracing::debug;

/// Create the Login Routes and return the Router
pub fn routes(mm: ModelManager) -> Router {
    // Create the Login Route with the POST method and the api_login handler
    Router::new()
        .route("/api/login", post(api_login_handler))
        .route("/api/logoff", post(api_logoff_handler))
        .with_state(mm) // mm is passed to the State that can be accessed in the handler with State(ModelManager)
}

// region:    --- Login
/// Login Handler that returns a JSON response with a status
async fn api_login_handler(
    State(mm): State<ModelManager>,
    cookies: Cookies,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_login_handler", "HANDLER");

    let LoginPayload {
        username,
        // password is the clear text password
        password: pwd_clear,
    } = payload;

    let root_ctx = Ctx::root_ctx();

    let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?;

    let user_id = user.id;
    // Check if the user has a password
    let Some(pwd) = user.password else {
        return Err(Error::LoginFailUserHasNoPwd { user_id });
    };

    pwd::validate_pwd(
        &ContentToHash {
            salt: user.password_salt,
            content: pwd_clear.clone(),
        },
        &pwd,
    )
    .map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

    // Set the token cookie
    web::set_token_cookie(&cookies, &user.username, user.token_salt)?;

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
// endregion: --- Login

// region:    --- Logoff
/// Logoff Handler that returns a JSON response with a status
async fn api_logoff_handler(
    cookies: Cookies,
    Json(payload): Json<LogoffPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_logoff_handler", "HANDLER");

    let should_logoff = payload.logoff;
    if should_logoff {
        let _ = remove_token_cookie(&cookies);
    }

    let body = Json(json!({
        "result": {
            "logoff": should_logoff
        }
    }));

    Ok(body)
}

/// Logoff Payload Struct for Deserialization
#[derive(Debug, Deserialize)]
struct LogoffPayload {
    logoff: bool,
}
// endregion: --- Logoff
