mod error;
pub mod mw_auth;
pub mod mw_res_map;
pub mod routes_login;
pub mod routes_static;

pub use self::error::ClientError;
pub use self::error::{Error, Result};

/// The name of the authentication token cookie
pub const AUTH_TOKEN: &str = "auth-token";
