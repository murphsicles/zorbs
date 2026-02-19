// src/config.rs
use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
}

impl Config {
    pub fn load() -> Self {
        dotenv().ok();
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        Self { port }
    }
}
