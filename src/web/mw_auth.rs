use crate::ctx::Ctx;
use crate::model::ModelController;
use crate::web::AUTH_TOKEN;
use crate::Error;
use crate::Result;
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

/// Middleware to require authentication for the request to continue to the next handler or middleware in the chain
pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    debug!(" {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    // Check if the Ctx was extracted correctly
    ctx?;

    // TODO: Token validation

    // Continue to the next handler or middleware in the chain
    Ok(next.run(req).await)
}

/// Middleware to resolve the context from the request cookies
pub async fn mw_ctx_resolver(
    _mc: State<ModelController>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!(" {:<12} - mw_ctx_resolver", "MIDDLEWARE");
    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let result_ctx = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((user_id, _exp, _sign)) => {
            // TODO: Token validation

            Ok(Ctx::new(user_id))
        }
        Err(e) => Err(e),
    };

    // Remove the cookie if something went wrong
    if result_ctx.is_err() && !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN));
    }

    // Store the ctx_result in the request extensions
    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

// region: Ctx Extractor
/// Implement the FromRequestParts trait for the Ctx type
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    /// Error type for the Ctx Extractor
    type Rejection = Error;

    /// Extract the Ctx from the request parts(Component parts of an HTTP Request)
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        debug!(" {:<12} - Ctx", "EXTRACTOR");

        // parts - HTTP Request except the body, headers
        // extensions - A map of extensions that can be used to share data between different components
        // get - Get an extension from the request
        // ok_or - Convert an Option to a Result
        // clone - Clone the Ctx
        parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::AuthFailCtxNotInRequestExtensions)?
            .clone()
    }
}
// endregion: Ctx Extractor

/// Parse a token from the cookies
fn parse_token(token: String) -> Result<(u64, String, String)> {
    // Parse the token into its parts
    let (_whole, _user_id, _exp, _sign) = regex_captures!(r#"^user-(\d+)\.(\w+)\.(\w+)"#, &token)
        .ok_or(Error::AuthFailTokenWrongFormat)?;
    // Parse the user_id into an u64
    let user_id = _user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenWrongFormat)?;
    // Return the user_id, exp, and sign
    Ok((user_id, _exp.to_string(), _sign.to_string()))
}
