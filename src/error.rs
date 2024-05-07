use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    LoginFail,

    // Model Error
    TicketDeleteFailIdNotFound { id: u64 },
}

// region: IntoResponse
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("--> {:<12} - Error - {error:?}", "HANDLER", error = self);
        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
    }
}
// endregion: IntoResponse
