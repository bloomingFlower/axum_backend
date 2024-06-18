mod consume;

use std::convert::Infallible;
use futures::{SinkExt, StreamExt};

use warp::Filter;
use warp::ws::{Message, WebSocket};
use tokio::sync::broadcast::{self, Receiver};
use crate::consume::{consume, MyMessage};

async fn handle_ws(
    ws: WebSocket,
    mut rx: Receiver<MyMessage>
) {
    let (mut ws_tx, _) = ws.split();
    
    while let Ok(msg) = rx.recv().await {
        let json_msg = serde_json::to_string(&msg).unwrap();
        ws_tx.send(Message::text(json_msg)).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = broadcast::channel(100);
    let msg: MyMessage = rx.recv().await.unwrap();
    tokio::spawn(async move { 
        consume(tx).await;
    });
    
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_msg(msg))
        .map(|ws: warp::ws::Ws, msg| {
            ws.on_upgrade(move |socket| handle_ws(socket, rx))
        });
    
    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await
}

fn with_msg(
    msg: MyMessage
) -> impl Filter<Extract = (MyMessage, )
    ,Error = Infallible> + Clone {
    warp::any().map(move || msg.clone())
}