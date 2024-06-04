//use anyhow::Result;
use color_eyre::eyre::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let span = tracing::info_span!("test_span");
    let _enter = span.enter();

    tracing::info!("Starting test 'test'");
    let ht = httpc_test::new_client("http://localhost:3000")?;
    ht.do_get("/index.html").await?.print().await?;
    let req_login = ht.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "password": "demo"
        }),
    );
    req_login.await?.print().await?;

    // ht.do_get("/hello").await?.print().await?;

    let mut task_ids: Vec<i64> = Vec::new();
    for i in 0..=4 {
        let req_create_task = ht.do_post(
            "/api/rpc",
            json!({
                "id": 1,
                "method": "task.create",
                "params": {
                    "data": {
                        "title": format!("test_create_ok title {i}")
                    }
                }
            }),
        );
        let result = req_create_task.await?;
        task_ids.push(result.json_value::<i64>("/result/id")?);
    }

    let req_update_task = ht.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "task.update",
            "params": {
                "id": task_ids[0],
                "data": {
                    "title": "test_create_ok title updated"
                }
            }
        }),
    );
    req_update_task.await?.print().await?;

    let req_delete_task = ht.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "task.delete",
            "params": {
                "id": task_ids[1]
            }
        }),
    );
    req_delete_task.await?.print().await?;

    let req_list_tasks = ht.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "task.list",
            "params": {
                "filters": [{
                    "title": {
                        "$startsWith": "test_create_ok"
                    }
                }, {
                    "id": {"$in": [task_ids[2], task_ids[3]]}
                }],
                "list_options": {
                    "order_bys": "!id"
                }
            }
        }),
    );
    req_list_tasks.await?.print().await?;

    let req_logoff = ht.do_post(
        "/api/logoff",
        json!({
            "logoff": true
        }),
    );
    // req_logoff.await?.print().await?;

    tracing::info!("Finished test 'test'");
    Ok(())
}
