use crate::log::log_request;
use crate::web;
use crate::web::mw_auth::CtxW;
use crate::web::routes_rpc::RpcInfo;
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, to_value};
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

/// Main response mapper
pub async fn main_response_mapper(
    ctx: Option<CtxW>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    let ctx = ctx.map(|ctx| ctx.0);

    debug!("{:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    let rpc_info = res.extensions().get::<Arc<RpcInfo>>();

    // -- Get the eventual response error.
    let web_error = res.extensions().get::<Arc<web::Error>>();
    let client_status_error = web_error.map(|se| se.client_status_and_error());

    // -- If client error, build the new response.
    let error_response = client_status_error
        .as_ref() // as_ref is used to avoid moving client_status_error(Option type)
        .map(|(status_code, client_error)| {
            let client_error = to_value(client_error).ok();
            let message = client_error.as_ref().and_then(|v| v.get("message"));
            let details = client_error.as_ref().and_then(|v| v.get("detail"));

            let client_error_body = json!({
                "error": {
                    "message": message,
                    "data": {
                        // FIXME: Request UUID should be set at the beginning of the request
                        "req_uuid": uuid.to_string(),
                        "detail": details,
                    }
                }
            });

            debug!("CLIENT ERROR BODY:\n{client_error_body}");

            // Build the new response from the client_error_body
            (*status_code, Json(client_error_body)).into_response()
        });

    // -- Build and log the server log line.
    let client_error = client_status_error.unzip().1;
    // TODO: Need to handle if log_request fail (but should not fail request)
    let _ = log_request(
        uuid,
        req_method,
        uri,
        rpc_info.map(Arc::as_ref),
        ctx,
        web_error.map(Arc::as_ref),
        client_error,
    )
    .await;

    debug!("\n");

    // unwrap_or_else is better than unwrap_or because it does not evaluate the default value if it is not needed.
    error_response.unwrap_or_else(|| res)
}
