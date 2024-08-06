mod config;

use axum::{
    http::{HeaderValue, Method},
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use futures::{stream, StreamExt};
use http::header;
use lib_consumer::{consume_stream, get_cached_message};
use lib_producer::produce_bitcoin_info;
use lib_producer::token::BitcoinInfo;
use rdkafka::message::Message;
use std::env;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::{convert::Infallible, path::PathBuf};
use tokio::sync::broadcast;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::sse_config;

use lib_core::model::redis_cache::RedisManager;
use std::sync::RwLock;

const REDIS_EXPIRATION: usize = 2700;
/// Local cache for Bitcoin information
struct LocalCache {
    bitcoin_info: Option<BitcoinInfo>,
}

/// /// Main function to initialize the SSE service
/// 1. Initialize tracing subscriber for logging
/// 2. Produce initial Bitcoin information
/// 3. Create a broadcast channel for Bitcoin information
/// 4. Spawn a task to consume data from Kafka and broadcast it
/// 5. Create and run the Axum application
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    // Initialize a Redis manager
    let redis_manager = RedisManager::initialize()
        .await
        .expect("Failed to initialize Redis manager");
    debug!("--> Main: Redis manager created");
    // Initialize a local cache
    let local_cache = Arc::new(RwLock::new(LocalCache { bitcoin_info: None }));
    debug!("--> Main: Local cache initialized");

    // Get Data from Kafka
    // Spawn a new task to consume data from Kafka and broadcast it
    let redis_manager_clone = redis_manager.clone();
    let local_cache_clone = local_cache.clone();
    tokio::spawn(async move {
        debug!("--> SSE Service::Kafka Consumer Task: Started");
        // Get the message stream from the Kafka consumer
        let message_stream = consume_stream("token").await;
        // Check if the message stream is Ok
        if let Ok(mut message_stream) = message_stream {
            debug!("--> SSE Service::Kafka Consumer Task: Message stream received");
            // Consume messages from the stream and send them to the broadcast channel
            while let Some(message_result) = message_stream.next().await {
                // Check if the message is Ok
                if let Ok(message) = message_result {
                    // Check if the message has a payload
                    if let Some(payload) = message.payload() {
                        debug!("--> SSE Service::Kafka Consumer Task: Received message payload");
                        if let Ok(bitcoin_info) =
                            parse_bitcoin_info(String::from_utf8_lossy(payload).to_string())
                        {
                            debug!(
                                "--> SSE Service::Kafka Consumer Task: Bitcoin info parsed: {:?}",
                                bitcoin_info
                            );
                            // Send the Bitcoin information to the broadcast channel
                            let _ = tx_clone.send(bitcoin_info.clone());
                            // Cache the message in Redis
                            if let Err(e) = redis_manager_clone
                                .set("bitcoin_info", &bitcoin_info, REDIS_EXPIRATION)
                                .await
                            {
                                error!("--> SSE Service::Kafka Consumer Task: Failed to cache Bitcoin info in Redis: {:?}", e);
                            } else {
                                debug!("--> SSE Service::Kafka Consumer Task: Bitcoin info cached in Redis");
                            }
                            // Cache the message in local memory
                            {
                                let mut cache = local_cache_clone.write().unwrap();
                                cache.bitcoin_info = Some(bitcoin_info);
                                debug!("--> SSE Service::Kafka Consumer Task: Bitcoin info cached locally");
                            }
                        } else {
                            error!("--> SSE Service::Kafka Consumer Task: Failed to parse Bitcoin info from payload");
                        }
                    }
                } else {
                    debug!("--> SSE Service::Kafka Consumer Task: Failed to receive message, checking caches");
                    // Check the local cache first
                    let local_bitcoin_info = {
                        let cache = local_cache_clone.read().unwrap();
                        cache.bitcoin_info.clone()
                    };

                    match local_bitcoin_info {
                        Some(bitcoin_info) => {
                            debug!("--> SSE Service::Kafka Consumer Task: Using Bitcoin info from local cache");
                            let _ = tx_clone.send(bitcoin_info);
                        }
                        None => {
                            debug!("--> SSE Service::Kafka Consumer Task: Local cache empty, checking Redis");
                            // If the local cache is empty, check Redis
                            match redis_manager_clone.get::<BitcoinInfo>("bitcoin_info").await {
                                Ok(Some(bitcoin_info)) => {
                                    debug!("--> SSE Service::Kafka Consumer Task: Bitcoin info retrieved from Redis");
                                    let _ = tx_clone.send(bitcoin_info.clone());
                                    // Update the local cache
                                    {
                                        let mut cache = local_cache_clone.write().unwrap();
                                        cache.bitcoin_info = Some(bitcoin_info);
                                        debug!("--> SSE Service::Kafka Consumer Task: Local cache updated with Redis data");
                                    }
                                }
                                Ok(None) => info!("--> SSE Service::Kafka Consumer Task: No cached Bitcoin info available in Redis"),
                                Err(e) => error!("--> SSE Service::Kafka Consumer Task: Failed to get cached Bitcoin info from Redis: {:?}", e),
                            }
                        }
                    }
                }
            }
        } else {
            error!("--> SSE Service::Kafka Consumer Task: Failed to get message stream");
            // If we couldn't get a message stream, try to use the cached message
            if let Some(cached_message) = get_cached_message().await {
                if let Ok(bitcoin_info) = parse_bitcoin_info(cached_message) {
                    info!(
                        "--> SSE Service::Kafka Consumer Task: Using cached Bitcoin info: {:?}",
                        bitcoin_info
                    );
                    let _ = tx_clone.send(bitcoin_info);
                    info!("--> SSE Service::Kafka Consumer Task: Cached Bitcoin info sent to broadcast channel");
                }
            }
        }
        debug!("--> SSE Service::Kafka Consumer Task: Completed");
    });

    // Create the Axum application
    let app = app(tx, redis_manager, local_cache);

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
fn parse_bitcoin_info(
    message: String,
) -> Result<BitcoinInfo, Box<dyn std::error::Error + Send + Sync>> {
    // Recieve a JSON string and parse it into a BitcoinInfo struct
    let bitcoin_info: BitcoinInfo = serde_json::from_str(&message)?;
    // Return the Bitcoin information
    Ok(bitcoin_info)
}

/// Create the Axum application
/// Support multiple clients with SSE
fn app(
    tx: Arc<broadcast::Sender<BitcoinInfo>>,
    redis_manager: Arc<RedisManager>,
    local_cache: Arc<RwLock<LocalCache>>,
) -> Router {
    // Serve static files from the assets directory
    // CARGO_MANIFEST_DIR is the path to the directory containing the Cargo.toml file
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    // Append index.html to the directory path if the request is for a directory
    let static_files_service = ServeDir::new(assets_dir).append_index_html_on_directories(true);
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(vec![
            "http://localhost:8080".parse::<HeaderValue>().unwrap(),
            "https://blog.yourrubber.duckdns.org"
                .parse::<HeaderValue>()
                .unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_credentials(true);

    // Build the application with a route
    Router::new()
        // Fallback to the static files service if the request is for a directory
        .fallback_service(static_files_service)
        // Route the request to the SSE handler
        .route(
            "/sse",
            get(move |user_agent| {
                sse_handler(
                    user_agent,
                    tx.clone(),
                    redis_manager.clone(),
                    local_cache.clone(),
                )
            }),
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
    redis_manager: Arc<RedisManager>,
    local_cache: Arc<RwLock<LocalCache>>,
) -> impl IntoResponse {
    debug!(
        "--> SSE Handler: New connection from user agent: {}",
        user_agent.as_str()
    );
    let rx = tx.subscribe();

    // Create a stream that first sends the cached data, then listens for updates
    let stream = stream::once(async move {
        debug!("--> SSE Handler: Checking caches for initial data");
        // Check the local cache first
        let local_bitcoin_info = {
            let cache = local_cache.read().unwrap();
            cache.bitcoin_info.clone()
        };

        match local_bitcoin_info {
            Some(bitcoin_info) => {
                debug!("--> SSE Handler: Using data from local cache");
                create_sse_event(bitcoin_info, "Sending cached update from local cache")
            }
            None => {
                debug!("--> SSE Handler: Local cache empty, checking Redis");
                // If the local cache is empty, check Redis
                match redis_manager.get::<BitcoinInfo>("bitcoin_info").await {
                    Ok(Some(bitcoin_info)) => {
                        debug!("--> SSE Handler: Data found in Redis, updating local cache");
                        // Update the local cache
                        {
                            let mut cache = local_cache.write().unwrap();
                            cache.bitcoin_info = Some(bitcoin_info.clone());
                        }
                        create_sse_event(bitcoin_info, "Sending cached update from Redis")
                    }
                    Ok(None) => {
                        debug!("--> SSE Handler: No data found in Redis");
                        create_waiting_event("No cached data available")
                    }
                    Err(e) => {
                        error!("--> SSE Handler: Redis error: {:?}", e);
                        create_error_event(format!("Failed to get cached data: {:?}", e))
                    }
                }
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

/// Helper functions
fn create_sse_event(bitcoin_info: BitcoinInfo, log_message: &str) -> Result<Event, Infallible> {
    debug!(
        "--> SSE Event Creator: Creating SSE event with message: {}",
        log_message
    );
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

    Ok::<Event, Infallible>(event)
}

fn create_waiting_event(message: &str) -> Result<Event, Infallible> {
    debug!(
        "--> SSE Event Creator: Creating waiting event with message: {}",
        message
    );
    let waiting_message = SSEMessage {
        status: "waiting".to_string(),
        data: None,
    };
    let waiting_str = serde_json::to_string(&waiting_message).unwrap();
    Ok::<Event, Infallible>(Event::default().data(waiting_str))
}

fn create_error_event(error_message: String) -> Result<Event, Infallible> {
    error!(
        "--> SSE Event Creator: Creating error event with message: {}",
        error_message
    );
    let error_message = SSEMessage {
        status: "error".to_string(),
        data: None,
    };
    let error_str = serde_json::to_string(&error_message).unwrap();
    Ok::<Event, Infallible>(Event::default().data(error_str))
}
