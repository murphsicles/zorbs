// src/utils.rs
use std::io::{self, Cursor, Read};
use flate2::read::GzDecoder;
use tar::Archive;
use toml::Value;
use crate::models::NewZorb;
use semver::Version;

const MAX_UPLOAD_SIZE: usize = 50 * 1024 * 1024; // 50 MB

const RESERVED_SCOPES: &[&str] = &[
    "google", "facebook", "meta", "xai", "x", "twitter", "apple", "microsoft", "amazon", "aws",
    "tesla", "nvidia", "intel", "amd", "oracle", "ibm", "openai", "anthropic", "deepmind",
    "zeta", "zorbs", "official", "admin", "root", "system", "registry", "youtube", "instagram",
    "whatsapp", "telegram", "tiktok", "netflix", "disney", "spotify", "github", "gitlab",
    "bitbucket", "adobe", "uber", "airbnb", "lyft", "samsung", "sony", "grok", "bitcoin",
    "btc", "bch", "bsv", "xrp", "sol", "solana", "bags", "bagsapp", "bagged", "crypto",
    "spacex", "nasa", "gov", "cia", "m15", "mi6", "gchq", "mod", "royal", "hrh", 
];

const BLOCKED_WORDS: &[&str] = &[
    "sex", "fuck", "porn", "pussy", "dick", "cunt", "asshole", "bitch", "shit", "nigger",
    "faggot", "whore", "slut", "cum", "rape", "pedo", "nazi", "hitler", "kkk", "nsfw", "xxx",
    "adult", "ass", "cock", "vagina", "penis", "kill", "murder", "suicide",
];

pub fn zorb_filename(name: &str, version: &str) -> String {
    let sanitized = name.replace('@', "").replace('/', "-").replace(' ', "-").to_lowercase();
    format!("{}-{}.zorb", sanitized, version)
}

pub fn validate_package_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Package name cannot be empty".to_string());
    }

    let lower_name = name.to_lowercase();

    // Hard block NSFW/profane anywhere in the name
    for word in BLOCKED_WORDS {
        if lower_name.contains(word) {
            return Err("Package name contains inappropriate content and is not allowed.".to_string());
        }
    }

    if name.starts_with('@') {
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() != 2 {
            return Err("Scoped name must be in format @scope/name".to_string());
        }

        let scope = parts[0].trim_start_matches('@').to_lowercase();
        let pkg = parts[1].to_lowercase();

        // Reserved brand scopes
        if RESERVED_SCOPES.contains(&scope.as_str()) {
            return Err(format!("The scope '@{}' is reserved. To claim it, please contact hi@zorbs.io", scope));
        }

        if scope.len() < 2 {
            return Err("Scope must be at least 2 characters long".to_string());
        }
        if !scope.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') || scope.starts_with('-') || scope.ends_with('-') {
            return Err("Scope may only contain alphanumeric characters, -, _ and must not start or end with -".to_string());
        }
        if !pkg.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') || pkg.starts_with('-') || pkg.ends_with('-') {
            return Err("Package name may only contain alphanumeric characters, -, _ and must not start or end with -".to_string());
        }
    } else {
        let pkg = lower_name;
        if !pkg.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') || pkg.starts_with('-') || pkg.ends_with('-') {
            return Err("Package name may only contain alphanumeric characters, -, _ and must not start or end with -".to_string());
        }

        // Also block reserved names for flat packages
        if RESERVED_SCOPES.contains(&pkg.as_str()) {
            return Err(format!("The name '{}' is reserved. To claim it, please contact hi@zorbs.io", name));
        }
    }

    Ok(())
}

pub fn validate_version(version: &str) -> Result<(), String> {
    match Version::parse(version) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Invalid semantic version: {}", version)),
    }
}

fn is_safe_path(path: &str) -> bool {
    !path.contains("..") &&
    !path.starts_with('/') &&
    !path.starts_with('\\') &&
    !path.contains("/../") &&
    !path.contains("\\..\\")
}

pub fn parse_zorb_toml(file_bytes: &[u8]) -> Result<NewZorb, String> {
    if file_bytes.len() > MAX_UPLOAD_SIZE {
        return Err(format!("Upload too large. Maximum size is {} MB", MAX_UPLOAD_SIZE / 1024 / 1024));
    }

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

        if !is_safe_path(&path) {
            return Err("Path traversal attempt detected in tarball".to_string());
        }

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

            validate_package_name(&name)?;
            validate_version(&version)?;

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
