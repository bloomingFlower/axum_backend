use crate::web::AUTH_TOKEN;
use crate::Error;
use crate::Result;
use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use lazy_regex::regex_captures;
use tower_cookies::Cookies;

pub async fn mw_require_auth(cookies: Cookies, req: Request<Body>, next: Next) -> Result<Response> {
    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    println!(
        "--> {:<12} - mw_require_auth - {auth_token:?}",
        "HANDLER",
        auth_token = auth_token
    );

    let (_user_id, _exp, _sign) = auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)?;

    // TODO: Token validation

    Ok(next.run(req).await)
}

/// Parse a toekn from the cookies
fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_whole, _user_id, _exp, _sign) = regex_captures!(r#"^user-(\d+)\.(\w+)\.(\w+)"#, &token)
        .ok_or(Error::AuthFailTokenWrongFormat)?;

    let user_id = _user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenWrongFormat)?;

    Ok((user_id, _exp.to_string(), _sign.to_string()))
}
