use crate::web::{set_token_cookie, AUTH_TOKEN};
use crate::web::{Error, Result};
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use lib_auth::token::{validate_web_token, Token};
use lib_core::ctx::Ctx;
use lib_core::model::user::{UserBmc, UserForAuth};
use lib_core::model::ModelManager;
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

#[allow(dead_code)] // For now, until we have the rpc.
pub async fn mw_require_auth(
    ctx: Result<CtxW>,
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolver(
    mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

    let ctx_ext_result = _ctx_resolve(mm, &cookies).await;

    if ctx_ext_result.is_err() && !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN))
    }

    // Store the ctx_ext_result in the request extension
    req.extensions_mut().insert(ctx_ext_result);

    Ok(next.run(req).await)
}

async fn _ctx_resolve(mm: State<ModelManager>, cookies: &Cookies) -> CtxExtResult {
    // Get Token String
    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(CtxExtError::TokenNotInCookie)?;

    // Parse Token
    let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

    // Get UserForAuth
    let user: UserForAuth = UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.identifier)
        .await
        .map_err(|e| CtxExtError::ModelAccessError(e.to_string()))?
        .ok_or(CtxExtError::UserNotFound)?;

    // Validate Token
    validate_web_token(&token, user.token_salt).map_err(|_| CtxExtError::FailValidate)?;

    // Update Token
    set_token_cookie(cookies, &user.username, user.token_salt)
        .map_err(|_| CtxExtError::CannotSetTokenCookie)?;

    // Create CtxExtResult
    Ctx::new(user.id)
        .map(CtxW)
        .map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// region:    --- Ctx Extractor
#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

// async_trait is used to define async trait that can be used in async functions
#[async_trait]
// This trait is used to extract the Ctx from the request parts which satisfies the S that Send and Sync constraints
// Send is used to move the ownership of the value to another thread
// Sync is used to safely share the value between threads
impl<S: Send + Sync> FromRequestParts<S> for CtxW {
    // Rejection is used to define the error type that can be returned by the from_request_parts function
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        debug!("{:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<CtxExtResult>()
            .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            .map_err(Error::CtxExt) // Return web error
    }
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    TokenNotInCookie,
    TokenWrongFormat,

    UserNotFound,
    ModelAccessError(String),
    FailValidate,
    CannotSetTokenCookie,

    CtxNotInRequestExt,
    CtxCreateFail(String),
}
// endregion: --- Ctx Extractor Result/Error
