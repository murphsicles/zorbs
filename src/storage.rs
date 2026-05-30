// src/storage.rs — Abstract storage layer: local filesystem or S3-compatible (Cloudflare R2)
// Falls back to local filesystem when cloud credentials are absent.
// Uses reqwest directly for S3 API calls (no extra crate needed).

use std::sync::Arc;

/// Unified storage backend for zorb packages.
pub enum StorageBackend {
    Local(LocalStorage),
    S3(S3Storage),
}

impl StorageBackend {
    /// Store bytes at the given key (e.g. `"crypto/bsv58-0.2.0.zorb"`).
    pub async fn store(&self, key: &str, data: &[u8]) -> Result<(), String> {
        match self {
            StorageBackend::Local(s) => s.store(key, data).await,
            StorageBackend::S3(s) => s.store(key, data).await,
        }
    }

    /// Return a download URL for the given key.
    pub fn download_url(&self, key: &str) -> String {
        match self {
            StorageBackend::Local(s) => s.download_url(key),
            StorageBackend::S3(s) => s.download_url(key),
        }
    }

    pub fn backend_name(&self) -> &str {
        match self {
            StorageBackend::Local(_) => "local",
            StorageBackend::S3(_) => "s3",
        }
    }
}

// ─── Local Filesystem ───────────────────────────────────────────────────────

pub struct LocalStorage {
    base_path: String,
}

impl LocalStorage {
    pub fn new(base_path: &str) -> Self {
        Self { base_path: base_path.to_string() }
    }

    async fn store(&self, key: &str, data: &[u8]) -> Result<(), String> {
        use std::path::Path;
        use tokio::fs;
        let full_path = format!("{}/{}", self.base_path, key);
        if let Some(parent) = Path::new(&full_path).parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| format!("mkdir: {}", e))?;
        }
        fs::write(&full_path, data).await
            .map_err(|e| format!("write: {}", e))
    }

    fn download_url(&self, key: &str) -> String {
        format!("/uploads/{}", key)
    }
}

// ─── S3-compatible (Cloudflare R2 / AWS S3 / MinIO) ───────────────────────

pub struct S3Storage {
    bucket: String,
    endpoint: String,
    public_url_base: String,
    access_key: String,
    secret_key: String,
    use_ssl: bool,
}

impl S3Storage {
    pub fn new(
        bucket: &str, endpoint: &str, public_url_base: &str,
        access_key: &str, secret_key: &str, use_ssl: bool,
    ) -> Self {
        Self {
            bucket: bucket.to_string(),
            endpoint: endpoint.to_string(),
            public_url_base: public_url_base.to_string(),
            access_key: access_key.to_string(),
            secret_key: secret_key.to_string(),
            use_ssl,
        }
    }

    /// S3 PUT object using MinIO Client (mc) with built-in AWS SigV4 support.
    /// mc's SigV4 implementation is battle-tested and works with all S3-compatible stores.
    async fn store(&self, key: &str, data: &[u8]) -> Result<(), String> {
        let mc_alias = "myminio";

        // Write data to a temp file for mc
        let tmpfile = format!("/tmp/zorb_upload_{}", std::process::id());
        tokio::fs::write(&tmpfile, data).await
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        // Build the S3 endpoint URL for mc's alias
        let scheme = if self.use_ssl { "https" } else { "http" };

        // First set up the alias, then upload to the correct key
        let setup_cmd = format!(
            "mc alias set {} {}://{} {} {} && mc cp {} {}/{}/{}",
            mc_alias,
            scheme,
            self.endpoint,
            self.access_key,
            self.secret_key,
            tmpfile,
            mc_alias,
            self.bucket,
            key
        );

        let output = tokio::process::Command::new("sh")
            .args(["-c", &setup_cmd])
            .output()
            .await
            .map_err(|e| format!("mc execution failed: {}", e))?;

        // Cleanup temp file
        let _ = tokio::fs::remove_file(&tmpfile).await;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(format!("S3 PUT failed (exit={}): {} {}", output.status, stdout.trim(), stderr.trim()))
        }
    }

    fn download_url(&self, key: &str) -> String {
        format!("{}/{}", self.public_url_base.trim_end_matches('/'), key)
    }
}

// ─── Factory ──────────────────────────────────────────────────────────────

/// Build the storage backend from environment configuration.
pub fn from_env() -> Arc<StorageBackend> {
    let bucket = std::env::var("R2_BUCKET").unwrap_or_default();
    let endpoint = std::env::var("R2_ENDPOINT").unwrap_or_default();
    let public_url = std::env::var("R2_PUBLIC_URL").unwrap_or_default();
    let access_key = std::env::var("R2_ACCESS_KEY_ID")
        .or_else(|_| std::env::var("AWS_ACCESS_KEY_ID"))
        .unwrap_or_default();
    let secret_key = std::env::var("R2_SECRET_ACCESS_KEY")
        .or_else(|_| std::env::var("AWS_SECRET_ACCESS_KEY"))
        .unwrap_or_default();

    if !bucket.is_empty() && !access_key.is_empty() {
        let use_ssl = std::env::var("R2_USE_SSL")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";
        let storage = S3Storage::new(&bucket, &endpoint, &public_url, &access_key, &secret_key, use_ssl);
        tracing::info!("Storage: S3-compatible ({})", endpoint);
        Arc::new(StorageBackend::S3(storage))
    } else {
        let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
        tracing::info!("Storage: local ({})", upload_dir);
        Arc::new(StorageBackend::Local(LocalStorage::new(&upload_dir)))
    }
}

// Crypto helper kept for reference; active S3 storage uses curl --aws-sigv4 directly
// (which has a battle-tested SigV4 implementation).
