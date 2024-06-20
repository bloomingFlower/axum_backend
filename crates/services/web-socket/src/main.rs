mod consume;
mod ws;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;

use consume::consume_msg;

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                println!("Received: {}", text);
                let formatted_response = format!("Echo: {}", text);
                socket
                    .send(Message::Text(formatted_response))
                    .await
                    .unwrap();
            }
            Message::Close(_) => {
                println!("Connection closed");
                break;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    // let (tx, _) = tokio::sync::broadcast::channel(10);
    // tokio::spawn(async move { consume_msg(tx).await });
    let app = Router::new().route("/send", get(ws_handler));

    let listener = TcpListener::bind("0.0.0.0:3001").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
