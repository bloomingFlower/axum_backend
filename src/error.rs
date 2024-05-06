use axum::response::{IntoResponse, Response};
use axum::http::{StatusCode};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    LoginFail,
}

// region: IntoResponse
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("--> {:<12} - Error - {error:?}", "HANDLER", error = self);
        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
    }
}
// endregion: IntoResponse