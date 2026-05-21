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

    /// S3 PUT object using direct HTTP PUT with AWS Signature V4 signing.
    async fn store(&self, key: &str, data: &[u8]) -> Result<(), String> {
        let scheme = if self.use_ssl { "https" } else { "http" };
        let url = format!("{}://{}/{}/{}", scheme, self.endpoint, self.bucket, key);
        let date = chrono::Utc::now().format("%Y%m%d").to_string();
        let datetime = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

        // Simple AWS SigV4 signing for S3 PUT
        let content_sha256 = sha256_hex(data);
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";
        let credential_scope = format!("{}/auto/s3/aws4_request", date);

        // StringToSign
        let algorithm = "AWS4-HMAC-SHA256";
        let canonical_request = format!(
            "PUT\n/{}/{}\n\nhost:{}.{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n\n{}\n{}",
            self.bucket, key,
            self.bucket, self.endpoint,
            content_sha256, datetime,
            signed_headers, content_sha256,
        );
        let hashed_cr = sha256_hex(canonical_request.as_bytes());
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm, datetime, credential_scope, hashed_cr,
        );

        // Signing key
        let signing_key = hmac_sha256(
            &hmac_sha256(
                &hmac_sha256(
                    &hmac_sha256(
                        format!("AWS4{}", self.secret_key).as_bytes(),
                        &date.as_bytes(),
                    ),
                    b"s3",
                ),
                b"aws4_request",
            ),
            b"aws4_request",
        );

        let signature = hex_lower(&hmac_sha256(&signing_key, string_to_sign.as_bytes()));

        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, self.access_key, credential_scope, signed_headers, signature,
        );

        let client = reqwest::Client::new();
        let resp = client
            .put(&url)
            // For virtual-hosted style: {bucket}.{endpoint}
            // For path-style: {endpoint}
            // MinIO uses path-style by default
            .header("Host", &self.endpoint)
            .header("x-amz-content-sha256", &content_sha256)
            .header("x-amz-date", &datetime)
            .header("Authorization", &authorization)
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| format!("S3 PUT request failed: {}", e))?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(format!("S3 PUT failed ({}): {}", status, body))
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

// ─── Crypto helpers ───────────────────────────────────────────────────────

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(data);
    hex::encode(hash)
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key).expect("HMAC key");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
