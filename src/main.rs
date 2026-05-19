// main.rs — Zorbs registry server binary (thin wrapper around library)

use zorbs::{build_app, config, db, state};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app_state = state::new();
    db::run_migrations(&app_state.db).await;

    let app = build_app(app_state);

    let listener = tokio::net::TcpListener::bind(config::addr())
        .await
        .expect("Failed to bind");
    tracing::info!("🚀 Zorbs registry listening on {}", config::addr());
    axum::serve(listener, app).await.unwrap();
}
