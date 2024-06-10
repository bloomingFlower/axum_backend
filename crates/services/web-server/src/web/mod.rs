mod error;
pub mod mw_auth;
pub mod mw_res_map;
pub mod routes_login;
pub mod routes_rpc;
pub mod routes_static;

pub use self::error::ClientError;
pub use self::error::{Error, Result};
use lib_auth::token::generate_web_token;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

/// The name of the authentication token cookie
pub const AUTH_TOKEN: &str = "auth-token";

/// Set the token cookie
fn set_token_cookie(cookies: &Cookies, user: &str, salt: Uuid) -> Result<()> {
    // Generate the web token
    let token = generate_web_token(user, salt)?;

    let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());
    // Javascript can't access the cookie
    cookie.set_http_only(true);
    // The cookie is sent over all requests
    cookie.set_path("/");

    cookies.add(cookie);

    Ok(())
}

/// Remove the token cookie
fn remove_token_cookie(cookies: &Cookies) -> Result<()> {
    let mut cookie = Cookie::from(AUTH_TOKEN);
    // The cookie path should be the same as the one set
    cookie.set_path("/");

    cookies.remove(cookie);

    Ok(())
}
