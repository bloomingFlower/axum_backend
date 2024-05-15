pub mod mw_auth;
pub mod routes_login;
pub(crate) mod routes_static;
pub mod routes_tickets;

/// The name of the authentication token cookie
pub const AUTH_TOKEN: &str = "auth-token";
