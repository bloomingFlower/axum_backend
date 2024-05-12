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

pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    println!("--> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    // TODO: Token validation

    Ok(next.run(req).await)
}

// region: Ctx Extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("--> {:<12} - Ctx", "EXTRACTOR");

        let cookies = parts.extract::<Cookies>().await.unwrap();

        let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

        let (user_id, _exp, _sign) = auth_token
            .ok_or(Error::AuthFailNoAuthTokenCookie)
            .and_then(parse_token)?; // FIXME: Implement a real token validation

        Ok(Ctx::new(user_id))
    }
}
// endregion: Ctx Extractor

/// Parse a toekn from the cookies
fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_whole, _user_id, _exp, _sign) = regex_captures!(r#"^user-(\d+)\.(\w+)\.(\w+)"#, &token)
        .ok_or(Error::AuthFailTokenWrongFormat)?;

    let user_id = _user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenWrongFormat)?;

    Ok((user_id, _exp.to_string(), _sign.to_string()))
}
