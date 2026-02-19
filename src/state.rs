/// src/state.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type AppState = Arc<Mutex<HashMap<String, Vec<u8>>>>;

pub fn new() -> AppState {
    Arc::new(Mutex::new(HashMap::new()))
}
