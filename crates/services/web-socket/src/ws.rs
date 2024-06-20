use axum::extract::ws::CloseFrame;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Close(Option<(CloseFrame<'static>)>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}
