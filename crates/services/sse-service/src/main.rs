mod config;

use axum::{
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use futures::{stream, StreamExt};
use lib_consumer::{consume_stream, get_cached_message};
use lib_producer::produce_bitcoin_info;
use lib_producer::token::BitcoinInfo;
use rdkafka::message::Message;
use std::env;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::{convert::Infallible, path::PathBuf};
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::sse_config;

/// /// Main function to initialize the SSE service
/// 1. Initialize tracing subscriber for logging
/// 2. Produce initial Bitcoin information
/// 3. Create a broadcast channel for Bitcoin information
/// 4. Spawn a task to consume data from Kafka and broadcast it
/// 5. Create and run the Axum application
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sse_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Produce Bitcoin Info
    let bitcoin_info = produce_bitcoin_info().await;
    info!("Bitcoin info: {:?}", bitcoin_info);

    // Create a broadcast channel
    // 100 is the buffer size
    // Multiple Receivers can be created using the same Sender
    let (tx, _) = broadcast::channel::<BitcoinInfo>(100);
    // Wrap the sender in an Arc to enable shared ownership
    let tx = Arc::new(tx);

    // Clone the sender to pass to the task
    // The reason for cloning is to create a new instance of the sender
    // that can be moved into the spawned task. This allows the task to
    // have its own reference to the sender, enabling it to send messages
    // independently of the original sender.
    let tx_clone = tx.clone();

    // Get Data from Kafka
    // Spawn a new task to consume data from Kafka and broadcast it
    tokio::spawn(async move {
        info!("--> SSE Service: Spawning task to consume data from Kafka");
        // Get the message stream from the Kafka consumer
        let message_stream = consume_stream("token").await;
        // Check if the message stream is Ok
        if let Ok(mut message_stream) = message_stream {
            info!("--> SSE Service: Message stream received");
            // Consume messages from the stream and send them to the broadcast channel
            while let Some(message_result) = message_stream.next().await {
                // Check if the message is Ok
                if let Ok(message) = message_result {
                    // Check if the message has a payload
                    if let Some(payload) = message.payload() {
                        // Parse the Bitcoin information from the payload
                        if let Ok(bitcoin_info) =
                            parse_bitcoin_info(String::from_utf8_lossy(payload).to_string())
                        {
                            info!("--> SSE Service: Bitcoin info parsed: {:?}", bitcoin_info);
                            // Send the Bitcoin information to the broadcast channel
                            let _ = tx_clone.send(bitcoin_info);
                            info!("--> SSE Service: Bitcoin info sent to broadcast channel");
                        }
                    }
                } else {
                    // If there's an error getting a new message, try to use the cached message
                    if let Some(cached_message) = get_cached_message().await {
                        if let Ok(bitcoin_info) = parse_bitcoin_info(cached_message) {
                            info!(
                                "--> SSE Service: Using cached Bitcoin info: {:?}",
                                bitcoin_info
                            );
                            let _ = tx_clone.send(bitcoin_info);
                            info!("--> SSE Service: Cached Bitcoin info sent to broadcast channel");
                        }
                    }
                }
            }
        } else {
            info!("--> SSE Service: Failed to get message stream, using cached message");
            // If we couldn't get a message stream, try to use the cached message
            if let Some(cached_message) = get_cached_message().await {
                if let Ok(bitcoin_info) = parse_bitcoin_info(cached_message) {
                    info!(
                        "--> SSE Service: Using cached Bitcoin info: {:?}",
                        bitcoin_info
                    );
                    let _ = tx_clone.send(bitcoin_info);
                    info!("--> SSE Service: Cached Bitcoin info sent to broadcast channel");
                }
            }
        }
        info!("--> SSE Service: Task to consume data from Kafka completed");
    });

    // Create the Axum application
    let app = app(tx);

    // Parsing the server URL
    let server_url = &sse_config().SSE_SERVER_URL;
    let addr = server_url
        .to_socket_addrs()
        .expect("Invalid server address")
        .next()
        .expect("Failed to resolve server address");

    info!("Binding to address: {:?}", addr);

    // TcpListener binding
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Logging the bound address
    info!("Listening on {}", listener.local_addr()?);

    // Run the server
    axum::serve(listener, app).await?;

    Ok(())
}

/// Parse the Bitcoin information from the message
fn parse_bitcoin_info(message: String) -> Result<BitcoinInfo, Box<dyn std::error::Error>> {
    // Recieve a JSON string and parse it into a BitcoinInfo struct
    let bitcoin_info: BitcoinInfo = serde_json::from_str(&message)?;
    // Return the Bitcoin information
    Ok(bitcoin_info)
}

/// Create the Axum application
/// Support multiple clients with SSE
fn app(tx: Arc<broadcast::Sender<BitcoinInfo>>) -> Router {
    // Serve static files from the assets directory
    // CARGO_MANIFEST_DIR is the path to the directory containing the Cargo.toml file
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    // Append index.html to the directory path if the request is for a directory
    let static_files_service = ServeDir::new(assets_dir).append_index_html_on_directories(true);
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the application with a route
    Router::new()
        // Fallback to the static files service if the request is for a directory
        .fallback_service(static_files_service)
        // Route the request to the SSE handler
        .route(
            "/sse",
            get(move |user_agent| sse_handler(user_agent, tx.clone())),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors) // Add CORS middleware
}

// BitcoinInfoWithDetails struct
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct BitcoinInfoWithDetails {
    price: f64,
    last_updated: String,
    high_24h: f64,
    low_24h: f64,
    price_change_24h: f64,
    price_change_percentage_24h: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct SSEMessage {
    status: String,
    data: Option<BitcoinInfoWithDetails>,
}

async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    tx: Arc<broadcast::Sender<BitcoinInfo>>,
) -> impl IntoResponse {
    info!("--> SSE Handler: `{}` connected", user_agent.as_str());

    let rx = tx.subscribe();

    // Create a stream that first sends the cached data, then listens for updates
    let stream = stream::once(async move {
        // Try to get the cached message first
        match get_cached_message().await {
            Some(cached_message) => match parse_bitcoin_info(cached_message) {
                Ok(bitcoin_info) => {
                    let info_with_details = BitcoinInfoWithDetails {
                        price: bitcoin_info.current_price,
                        last_updated: bitcoin_info.last_updated,
                        high_24h: bitcoin_info.high_24h,
                        low_24h: bitcoin_info.low_24h,
                        price_change_24h: bitcoin_info.price_change_24h,
                        price_change_percentage_24h: bitcoin_info.price_change_percentage_24h,
                    };

                    let sse_message = SSEMessage {
                        status: "success".to_string(),
                        data: Some(info_with_details),
                    };

                    let info_str = serde_json::to_string(&sse_message).unwrap();
                    let event = Event::default().data(info_str);

                    info!("--> SSE Handler: Sending cached update: {:?}", sse_message);
                    Ok::<Event, Infallible>(event)
                }
                Err(e) => {
                    info!("--> SSE Handler: Failed to parse cached message: {:?}", e);
                    let waiting_message = SSEMessage {
                        status: "waiting".to_string(),
                        data: None,
                    };
                    let waiting_str = serde_json::to_string(&waiting_message).unwrap();
                    Ok::<Event, Infallible>(Event::default().data(waiting_str))
                }
            },
            None => {
                info!("--> SSE Handler: No cached data available");
                let waiting_message = SSEMessage {
                    status: "waiting".to_string(),
                    data: None,
                };
                let waiting_str = serde_json::to_string(&waiting_message).unwrap();
                Ok::<Event, Infallible>(Event::default().data(waiting_str))
            }
        }
    })
    .chain(stream::unfold(rx, move |mut rx| async move {
        match rx.recv().await {
            Ok(bitcoin_info) => {
                let info_with_details = BitcoinInfoWithDetails {
                    price: bitcoin_info.current_price,
                    last_updated: bitcoin_info.last_updated,
                    high_24h: bitcoin_info.high_24h,
                    low_24h: bitcoin_info.low_24h,
                    price_change_24h: bitcoin_info.price_change_24h,
                    price_change_percentage_24h: bitcoin_info.price_change_percentage_24h,
                };

                let sse_message = SSEMessage {
                    status: "success".to_string(),
                    data: Some(info_with_details),
                };

                let info_str = serde_json::to_string(&sse_message).unwrap();
                let event = Event::default().data(info_str);

                info!("--> SSE Handler: Sending update: {:?}", sse_message);
                Some((Ok::<Event, Infallible>(event), rx))
            }
            Err(e) => {
                info!("--> SSE Handler: Error receiving message: {:?}", e);
                let error_message = SSEMessage {
                    status: "error".to_string(),
                    data: None,
                };
                let error_str = serde_json::to_string(&error_message).unwrap();
                Some((
                    Ok::<Event, Infallible>(Event::default().data(error_str)),
                    rx,
                ))
            }
        }
    }));

    let sse = Sse::new(stream).keep_alive(KeepAlive::default());

    (
        [(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("text/event-stream"),
        )],
        sse,
    )
}

#[cfg(test)]
mod tests {
    // Import necessary modules for testing
    use super::*;
    use eventsource_stream::Eventsource;
    use futures::StreamExt;
    use tokio::net::TcpListener;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn integration_test() {
        // A helper function that spawns our application in the background
        async fn spawn_app(host: impl Into<String>) -> String {
            let host = host.into();
            // Bind to localhost at the port 0, which will let the OS assign an available port to us
            let listener = TcpListener::bind(format!("{}:0", host)).await.unwrap();
            // Retrieve the port assigned to us by the OS
            let port = listener.local_addr().unwrap().port();
            let (tx, _) = broadcast::channel::<BitcoinInfo>(100);
            let tx = Arc::new(tx);
            tokio::spawn(async {
                axum::serve(listener, app(tx)).await.unwrap();
            });
            // Returns address (e.g. http://127.0.0.1{random_port})
            format!("http://{}:{}", host, port)
        }
        let listening_url = spawn_app("127.0.0.1").await;

        // Create a new client to connect to the SSE endpoint
        let mut event_stream = reqwest::Client::new()
            .get(&format!("{}/sse", listening_url))
            .header("User-Agent", "integration_test")
            .send()
            .await
            .unwrap()
            .bytes_stream()
            .eventsource()
            .take(1);

        // Collect event data from the stream
        let mut event_data: Vec<String> = vec![];
        while let Some(event) = event_stream.next().await {
            match event {
                Ok(event) => {
                    // break the loop at the end of SSE stream
                    if event.data == "[DONE]" {
                        break;
                    }

                    event_data.push(event.data);
                }
                Err(_) => {
                    panic!("Error in event stream");
                }
            }
        }

        // Assert that the first event data is as expected
        assert!(event_data[0].contains("BitcoinInfo"));
    }
}
