use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub upload_dir: String,
}

impl Config {
    pub fn load() -> Self {
        dotenv().ok();
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let upload_dir = env::var("UPLOAD_DIR")
            .unwrap_or_else(|_| "uploads".to_string());
        Self { port, database_url, upload_dir }
    }
}
