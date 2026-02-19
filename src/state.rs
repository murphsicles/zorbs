use std::collections::HashMap;
use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::Mutex;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub packages: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

pub async fn new(config: &Config) -> Arc<AppState> {
    let db = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Failed to run migrations");
    Arc::new(AppState {
        db,
        packages: Arc::new(Mutex::new(HashMap::new())),
    })
}
