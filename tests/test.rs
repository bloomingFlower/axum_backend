use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test() -> Result<()> {
    let ht = httpc_test::new_client("http://localhost:3000")?;
    ht.do_get("/hello?name=JYY").await?.print().await?;
    ht.do_get("/hello2/JYY2").await?.print().await?;

    let req_login = ht.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "password": "demo1"
        })
    );
    req_login.await?.print().await?;

    Ok(())
}