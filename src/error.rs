use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Result type for this application with the Error type
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for this application
#[derive(Debug)]
pub enum Error {
    // Login Error
    LoginFail,

    // Auth Error
    AuthFailNoAuthTokenCookie,
    AuthFailTokenWrongFormat,

    // Model Error
    TicketDeleteFailIdNotFound { id: u64 },
}

// region: IntoResponse
/// Implement the IntoResponse trait for the Error type
impl IntoResponse for Error {
    /// Convert the Error type into a Response
    fn into_response(self) -> Response {
        println!("--> {:<12} - Error - {error:?}", "HANDLER", error = self);
        // Return a 500 Internal Server Error with the error message
        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
    }
}
// endregion: IntoResponse
