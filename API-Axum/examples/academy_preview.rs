//! Preview local da Academy, sem inicializar banco, carteiras ou mesas do lobby.
//! Executar no WSL: cargo run --example academy_preview. Porta fixa de loopback.
use axum::{extract::DefaultBodyLimit, routing::post, Router};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/academy/play", post(poker_api::academy::play))
        .layer(DefaultBodyLimit::max(16 * 1024));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3188")
        .await
        .expect("Porta local 3188 disponível");
    println!("Academy preview: http://127.0.0.1:3188 (sem banco ou carteira)");
    axum::serve(listener, app).await.expect("Servidor local");
}
