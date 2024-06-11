mod error;
mod params;
mod task_rpc;

pub use self::error::{Error, Result};
use params::*;

use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
use serde::Deserialize;
use serde_json::{from_value, to_value, Value};
use task_rpc::{create_task, delete_task, list_tasks, update_task};

/// JSON-RPC 2.0 Request
#[derive(Deserialize)]
pub struct RpcRequest {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// Define the macro to execute the RPC function
macro_rules! exec_rpc_fn {
    // With Params
    ($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {{
        let rpc_fn_name = stringify!($rpc_fn);
        let params = $rpc_params.ok_or(Error::RpcMissingParams {
            rpc_method: rpc_fn_name.to_string(),
        })?;
        let params = from_value(params).map_err(|_| Error::RpcFailJsonParams {
            rpc_method: rpc_fn_name.to_string(),
        })?;
        $rpc_fn($ctx, $mm, params).await.map(to_value)??
    }};

    // Without Params
    ($rpc_fn:expr, $ctx:expr, $mm:expr) => {
        $rpc_fn($ctx, $mm).await.map(to_value)??
    };
}

pub async fn exec_rpc(ctx: Ctx, mm: ModelManager, rpc_req: RpcRequest) -> Result<Value> {
    let rpc_method = rpc_req.method;
    let rpc_params = rpc_req.params;

    // Actually, these RPC methods are not appropriate for the RPC API because they are CRUD operations.
    // RESTful API is more suitable for CRUD operations.
    let result_json = match rpc_method.as_str() {
        "task.create" => exec_rpc_fn!(create_task, ctx, mm, rpc_params),
        "task.list" => exec_rpc_fn!(list_tasks, ctx, mm, rpc_params),
        "task.update" => exec_rpc_fn!(update_task, ctx, mm, rpc_params),
        "task.delete" => exec_rpc_fn!(delete_task, ctx, mm, rpc_params),
        _ => return Err(Error::RpcMethodUnknown(rpc_method)),
    };

    // The benefit of using JSON-RPC is that the response is always in the same format and the client can easily parse it.
    Ok(result_json)
}
