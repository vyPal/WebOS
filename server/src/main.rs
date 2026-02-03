use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{HeaderValue, Response},
    middleware::{self, Next},
    routing::get,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route_service("/", ServeFile::new("./server/static/index.html"))
        .fallback_service(ServeDir::new("./server/static"))
        .layer(middleware::from_fn(add_security_headers))
        .layer(TraceLayer::new_for_http());

    let addr = "0.0.0.0:3000";
    tracing::info!("listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn add_security_headers(request: Request, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    headers.insert(
        "Cross-Origin-Opener-Policy",
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        "Cross-Origin-Embedder-Policy",
        HeaderValue::from_static("require-corp"),
    );

    response
}
