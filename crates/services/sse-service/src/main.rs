use axum::{
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use futures::stream::{self, Stream};
use std::{convert::Infallible, path::PathBuf, time::Duration};
use tokio_stream::StreamExt as _;
use tower_http::{services::ServeDir, trace::TraceLayer};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use lib_producer::produce_bitcoin_info;
use lib_producer::token::BitcoinInfo;
use lib_consumer::consume;


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_sse=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let bitcoin_info = produce_bitcoin_info().await;
    info!("Bitcoin info: {:?}", bitcoin_info);

    // Create a broadcast channel
    let (tx, _) = broadcast::channel::<BitcoinInfo>(100);
    let tx = Arc::new(tx);

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut stream = consume("token").await;
        while let Some(message) = stream.next().await {
        if let Ok(bitcoin_info) = parse_bitcoin_info(message) {
                let _ = tx_clone.send(bitcoin_info);
            }
        }
    });

    let app = app(tx);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

fn parse_bitcoin_info(message: String) -> Result<BitcoinInfo, Box<dyn std::error::Error>> {
    // 메시지를 BitcoinInfo 구조체로 파싱
    let bitcoin_info: BitcoinInfo = serde_json::from_str(&message)?;
    Ok(bitcoin_info)
}

fn app(tx: Arc<broadcast::Sender<BitcoinInfo>>) -> Router {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let static_files_service = ServeDir::new(assets_dir).append_index_html_on_directories(true);
    // build application with a route
    Router::new()
        .fallback_service(static_files_service)
        .route("/sse", get(move |user_agent| sse_handler(user_agent, tx.clone())))
        .layer(TraceLayer::new_for_http())
}

async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    tx: Arc<broadcast::Sender<BitcoinInfo>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("`{}` connected", user_agent.as_str());

    let mut rx = tx.subscribe();

    let stream = stream::unfold((), move |_| async move {
        match rx.recv().await {
            Ok(bitcoin_info) => {
                let event = Event::default().data(format!("Bitcoin Price: {}", bitcoin_info.current_price));
                Some((Ok(event), ()))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

#[cfg(test)]
mod tests {
    use eventsource_stream::Eventsource;
    use tokio::net::TcpListener;

    use super::*;

    #[tokio::test]
    async fn integration_test() {
        // A helper function that spawns our application in the background
        async fn spawn_app(host: impl Into<String>) -> String {
            let host = host.into();
            // Bind to localhost at the port 0, which will let the OS assign an available port to us
            let listener = TcpListener::bind(format!("{}:0", host)).await.unwrap();
            // Retrieve the port assigned to us by the OS
            let port = listener.local_addr().unwrap().port();
            tokio::spawn(async {
                axum::serve(listener, app()).await.unwrap();
            });
            // Returns address (e.g. http://127.0.0.1{random_port})
            format!("http://{}:{}", host, port)
        }
        let listening_url = spawn_app("127.0.0.1").await;

        let mut event_stream = reqwest::Client::new()
            .get(&format!("{}/sse", listening_url))
            .header("User-Agent", "integration_test")
            .send()
            .await
            .unwrap()
            .bytes_stream()
            .eventsource()
            .take(1);

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

        assert!(event_data[0] == "hi!");
    }
}
