use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::rpc::{exec_rpc, RpcInfo, RpcRequest};
use crate::web::Result;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::debug;

/// Routes for JSON-RPC 2.0
pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(mm)
}

/// JSON-RPC 2.0 Handler
async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    let rpc_info = RpcInfo {
        id: rpc_req.id.clone(),
        method: rpc_req.method.clone(),
    };

    let mut res = _rpc_handler(ctx, mm, rpc_req).await.into_response();
    res.extensions_mut().insert(Arc::new(rpc_info));

    res
}

/// RPC Handler
async fn _rpc_handler(ctx: Ctx, mm: ModelManager, rpc_req: RpcRequest) -> Result<Json<Value>> {
    let rpc_method = rpc_req.method.clone();
    let rpc_id = rpc_req.id.clone();

    debug!("{:<12} - _rpc_handler - method: {rpc_method}", "HANDLER");

    let result = exec_rpc(ctx, mm, rpc_req).await?;

    // The benefit of using JSON-RPC is that the response is always in the same format and the client can easily parse it.
    let body_response = json!({
        "id": rpc_id,
        "result": result,
    });

    Ok(Json(body_response))
}
