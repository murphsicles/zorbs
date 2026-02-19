use std::env;

pub fn addr() -> String {
    env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
}

pub fn database_url() -> String {
    env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env")
}

pub fn upload_dir() -> String {
    env::var("UPLOAD_DIR")
        .unwrap_or_else(|_| "uploads".to_string())
}
