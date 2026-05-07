use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Zorb {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub owner_id: Option<Uuid>,
    pub downloads: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dependencies: JsonValue,
    pub readme: Option<String>,
}

impl Zorb {
    pub fn dependencies_map(&self) -> HashMap<String, String> {
        match &self.dependencies {
            JsonValue::Object(map) => {
                map.iter().map(|(k, v)| {
                    let v_str = v.as_str().unwrap_or("*").to_string();
                    (k.clone(), v_str)
                }).collect()
            }
            _ => HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NewZorb {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub dependencies: JsonValue,
    pub readme: Option<String>,
}
