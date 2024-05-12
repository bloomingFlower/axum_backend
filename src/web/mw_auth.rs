use crate::ctx::Ctx;
use crate::web::AUTH_TOKEN;
use crate::Error;
use crate::Result;
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::RequestPartsExt;
use lazy_regex::regex_captures;
use tower_cookies::Cookies;

/// Middleware to require authentication for the request to continue to the next handler or middleware in the chain
pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    println!("--> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    // Check if the Ctx was extracted correctly
    ctx?;

    // TODO: Token validation

    // Continue to the next handler or middleware in the chain
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
        println!("--> {:<12} - Ctx", "EXTRACTOR");
        // Extract the Cookies from the request parts
        let cookies = parts.extract::<Cookies>().await.unwrap();
        // Extract the AUTH_TOKEN cookie from the cookies
        let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());
        // Parse the token into its parts
        let (user_id, _exp, _sign) = auth_token
            .ok_or(Error::AuthFailNoAuthTokenCookie)
            .and_then(parse_token)?; // FIXME: Implement a real token validation
                                     // Create a new Ctx with the user_id and return it
        Ok(Ctx::new(user_id))
    }
}
// endregion: Ctx Extractor

/// Parse a token from the cookies
fn parse_token(token: String) -> Result<(u64, String, String)> {
    // Parse the token into its parts
    let (_whole, _user_id, _exp, _sign) = regex_captures!(r#"^user-(\d+)\.(\w+)\.(\w+)"#, &token)
        .ok_or(Error::AuthFailTokenWrongFormat)?;
    // Parse the user_id into a u64
    let user_id = _user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenWrongFormat)?;
    // Return the user_id, exp, and sign
    Ok((user_id, _exp.to_string(), _sign.to_string()))
}
