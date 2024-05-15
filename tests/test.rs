//use anyhow::Result;
use color_eyre::eyre::Result;
use serde_json::json;

#[tokio::test]
async fn test() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let span = tracing::info_span!("test_span");
    let _enter = span.enter();

    tracing::info!("Starting test 'test'");
    let ht = httpc_test::new_client("http://localhost:3000")?;
    ht.do_get("/hello?name=JYY").await?.print().await?;
    ht.do_get("/hello2/JYY2").await?.print().await?;
    ht.do_get("/index.html").await?.print().await?;
    let req_login = ht.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "password": "demo"
        }),
    );
    req_login.await?.print().await?;

    ht.do_get("/hello2/JYY2").await?.print().await?;

    let req_create_ticket = ht.do_post(
        "/api/tickets",
        json!({
            "title": "Ticket 1"
        }),
    );
    req_create_ticket.await?.print().await?;

    ht.do_delete("/api/tickets/1").await?.print().await?;

    ht.do_get("/api/tickets").await?.print().await?;

    tracing::info!("Finished test 'test'");

    Ok(())
}
