// src/utils.rs
use std::io::{self, Cursor, Read};
use flate2::read::GzDecoder;
use tar::Archive;
use toml::Value;
use crate::models::NewZorb;

pub fn zorb_filename(name: &str, version: &str) -> String {
    let sanitized = name.replace('@', "").replace('/', "-").replace(' ', "-").to_lowercase();
    format!("{}-{}.zorb", sanitized, version)
}

pub fn parse_zorb_toml(file_bytes: &[u8]) -> Result<NewZorb, String> {
    let cursor = Cursor::new(file_bytes);
    let decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(decoder);

    let mut entries = match archive.entries() {
        Ok(e) => e,
        Err(e) => return Err(format!("Failed to read tar archive: {}", e)),
    };

    for entry_result in entries {
        let mut entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = match entry.path() {
            Ok(p) => p.to_string_lossy().into_owned(),
            Err(_) => continue,
        };

        if path.ends_with("zorb.toml") || path.ends_with("Zorb.toml") {
            let mut content = String::new();
            if let Err(e) = entry.read_to_string(&mut content) {
                return Err(format!("Failed to read zorb.toml content: {}", e));
            }

            let parsed: Value = match toml::from_str(&content) {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed to parse zorb.toml: {}", e)),
            };

            let package = match parsed.get("package").and_then(Value::as_table) {
                Some(p) => p,
                None => return Err("Missing [package] section in zorb.toml".to_string()),
            };

            let name = match package.get("name").and_then(Value::as_str) {
                Some(n) => n.to_string(),
                None => return Err("Missing 'name' field in zorb.toml [package]".to_string()),
            };

            let version = match package.get("version").and_then(Value::as_str) {
                Some(v) => v.to_string(),
                None => return Err("Missing 'version' field in zorb.toml [package]".to_string()),
            };

            let description = package.get("description").and_then(Value::as_str).map(str::to_string);
            let license = package.get("license").and_then(Value::as_str).map(str::to_string);
            let repository = package.get("repository").and_then(Value::as_str).map(str::to_string);

            return Ok(NewZorb {
                name,
                version,
                description,
                license,
                repository,
            });
        }
    }

    Err("No zorb.toml found in the uploaded tarball".to_string())
}
