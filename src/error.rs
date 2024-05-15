use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::debug;

/// Result type for this application with the Error type
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for this application
#[derive(Debug, Clone, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // Login Error
    LoginFail,

    // Auth Error
    AuthFailNoAuthTokenCookie,
    AuthFailTokenWrongFormat,
    AuthFailCtxNotInRequestExtensions,

    // Model Error
    TicketDeleteFailIdNotFound { id: u64 },

    // Config Error
    ConfigMissingEnvVar { name: &'static str },
}

// region: IntoResponse
/// Implement the IntoResponse trait for the Error type
impl IntoResponse for Error {
    /// Convert the Error type into a Response
    fn into_response(self) -> Response {
        debug!(" {:<12} - Error - {error:?}", "HANDLER", error = self);
        // Return a 500 Internal Server Error with the error message

        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        response.extensions_mut().insert(self);

        response
    }
}
// endregion: IntoResponse

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        // #[allow(unreachable_patterns)]
        match self {
            Self::LoginFail => (StatusCode::UNAUTHORIZED, ClientError::LOGIN_FAIL),

            Self::AuthFailNoAuthTokenCookie
            | Self::AuthFailTokenWrongFormat
            | Self::AuthFailCtxNotInRequestExtensions => {
                (StatusCode::UNAUTHORIZED, ClientError::NO_AUTH)
            }

            _ => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVER_ERROR),
        }
    }
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    INVALID_PARAMS,
    SERVER_ERROR,
}
