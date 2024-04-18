use axum::Router;
use axum::routing::get;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    // 서버 주소를 설정합니다.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // 라우터를 생성하고 엔드포인트를 설정합니다.
    let app = Router::new().route("/", get(handler));

    // 서비스 빌더를 사용하여 미들웨어를 추가하고 서버를 실행합니다.
    let svc = ServiceBuilder::new().layer(TraceLayer::new_for_http()).service(app);

    axum::Server::bind(&addr)
        .serve(svc)
        .await
        .unwrap();
}
