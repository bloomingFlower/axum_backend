mod consume;
mod ws;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tokio::signal;
use tokio::net::TcpListener;
use tracing::debug;
use tracing_subscriber::EnvFilter;

use consume::consume_msg;

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                debug!("Received: {}", text);
                let formatted_response = format!("Echo: {}", text);
                socket
                    .send(Message::Text(formatted_response))
                    .await
                    .unwrap();
            }
            Message::Close(_) => {
                debug!("Connection closed");
                break;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt() // For standard output
        .without_time() // For local development(simplicity)
        .with_target(false) // For local development(simplicity)
        .with_env_filter(EnvFilter::from_default_env()) // set log level(config.toml)
        .init();
    // let (tx, _) = tokio::sync::broadcast::channel(10);
    // tokio::spawn(async move { consume_msg(tx).await });
    let app = Router::new().route("/send", get(ws_handler));

    let listener = TcpListener::bind("0.0.0.0:3001").await.unwrap();

    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
