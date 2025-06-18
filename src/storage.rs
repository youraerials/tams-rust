use crate::config::MediaStorageConfig;
use crate::error::{TamsError, TamsResult};
use crate::models::{GetUrl, StorageObject};
use chrono::{DateTime, Duration, Utc};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

#[derive(Clone)]
pub struct MediaStorage {
    config: MediaStorageConfig,
    public_base_url: String,
}

impl MediaStorage {
    pub fn new(config: MediaStorageConfig, public_base_url: String) -> TamsResult<Self> {
        Ok(MediaStorage {
            config,
            public_base_url,
        })
    }

    pub async fn ensure_directories(&self) -> TamsResult<()> {
        fs::create_dir_all(&self.config.base_path).await?;
        fs::create_dir_all(&self.config.temp_path).await?;
        Ok(())
    }

    pub async fn get_upload_url(&self, object_id: &str, _content_type: Option<&str>) -> TamsResult<String> {
        // In a real implementation, this would generate a presigned URL
        // For now, return a simple URL that points to our upload endpoint
        Ok(format!("{}/upload/{}", self.public_base_url, object_id))
    }

    pub async fn store_file(&self, object_id: &str, content: &[u8]) -> TamsResult<PathBuf> {
        let file_path = self.config.base_path.join(object_id);
        
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        fs::write(&file_path, content).await?;
        Ok(file_path)
    }

    pub async fn get_file_path(&self, object_id: &str) -> PathBuf {
        self.config.base_path.join(object_id)
    }

    pub async fn delete_file(&self, object_id: &str) -> TamsResult<()> {
        let file_path = self.get_file_path(object_id).await;
        if file_path.exists() {
            fs::remove_file(file_path).await?;
        }
        Ok(())
    }

    pub async fn file_exists(&self, object_id: &str) -> bool {
        self.get_file_path(object_id).await.exists()
    }

    pub fn get_public_url(&self, object_id: &str) -> String {
        format!("{}/media/{}", self.public_base_url, object_id)
    }

    /// Generate storage objects for new media uploads
    pub async fn allocate_storage(&self, count: u32, object_ids: Option<Vec<String>>) -> TamsResult<Vec<StorageObject>> {
        let mut objects = Vec::new();

        if let Some(ids) = object_ids {
            // Use provided object IDs
            for object_id in ids {
                self.validate_object_id(&object_id)?;
                let storage_obj = self.create_storage_object(object_id).await?;
                objects.push(storage_obj);
            }
        } else {
            // Generate new object IDs
            for _ in 0..count {
                let object_id = self.generate_object_id();
                let storage_obj = self.create_storage_object(object_id).await?;
                objects.push(storage_obj);
            }
        }

        Ok(objects)
    }

    /// Create a storage object with presigned upload URL
    async fn create_storage_object(&self, object_id: String) -> TamsResult<StorageObject> {
        let file_path = self.get_object_path(&object_id);
        
        // Ensure the parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Generate a presigned PUT URL (for our local implementation, this is a special endpoint)
        let put_url = format!("{}/objects/{}/upload", self.public_base_url, object_id);
        
        // URL expires in 1 hour
        let expires_at = Utc::now() + Duration::hours(1);

        Ok(StorageObject {
            object_id,
            put_url,
            put_headers: None,
            expires_at: Some(expires_at),
        })
    }

    /// Generate download URLs for existing objects
    pub async fn generate_get_urls(&self, object_id: &str, labels: Option<Vec<String>>) -> TamsResult<Vec<GetUrl>> {
        let file_path = self.get_object_path(object_id);
        
        if !file_path.exists() {
            return Err(TamsError::ObjectNotFound {
                object_id: object_id.to_string(),
            });
        }

        let mut urls = Vec::new();
        
        // Generate primary download URL
        let url = format!("{}/objects/{}/download", self.public_base_url, object_id);
        let expires_at = Utc::now() + Duration::hours(24); // URLs expire in 24 hours
        
        urls.push(GetUrl {
            url,
            label: None,
            expires_at: Some(expires_at),
        });

        // If specific labels are requested, generate labeled URLs
        if let Some(labels) = labels {
            for label in labels {
                let labeled_url = format!("{}/objects/{}/download?label={}", self.public_base_url, object_id, label);
                urls.push(GetUrl {
                    url: labeled_url,
                    label: Some(label),
                    expires_at: Some(expires_at),
                });
            }
        }

        Ok(urls)
    }

    /// Store media data for an object
    pub async fn store_object(&self, object_id: &str, data: Vec<u8>) -> TamsResult<()> {
        if data.len() as u64 > self.config.max_file_size {
            return Err(TamsError::FileTooLarge {
                max_size: self.config.max_file_size,
            });
        }

        self.validate_object_id(object_id)?;
        
        let file_path = self.get_object_path(object_id);
        
        // Ensure the parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write to a temporary file first, then rename for atomicity
        let temp_path = self.get_temp_path(&format!("{}.tmp", object_id));
        let mut temp_file = fs::File::create(&temp_path).await?;
        temp_file.write_all(&data).await?;
        temp_file.sync_all().await?;
        drop(temp_file);

        // Atomic rename
        fs::rename(&temp_path, &file_path).await?;

        tracing::info!("Stored object {} ({} bytes)", object_id, data.len());
        Ok(())
    }

    /// Retrieve media data for an object
    pub async fn get_object(&self, object_id: &str) -> TamsResult<Vec<u8>> {
        self.validate_object_id(object_id)?;
        
        let file_path = self.get_object_path(object_id);
        
        if !file_path.exists() {
            return Err(TamsError::ObjectNotFound {
                object_id: object_id.to_string(),
            });
        }

        let mut file = fs::File::open(&file_path).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;

        Ok(data)
    }

    /// Get object metadata (size, MIME type)
    pub async fn get_object_metadata(&self, object_id: &str) -> TamsResult<(u64, Option<String>)> {
        self.validate_object_id(object_id)?;
        
        let file_path = self.get_object_path(object_id);
        
        if !file_path.exists() {
            return Err(TamsError::ObjectNotFound {
                object_id: object_id.to_string(),
            });
        }

        let metadata = fs::metadata(&file_path).await?;
        let size = metadata.len();
        
        // Guess MIME type from file extension or content
        let mime_type = mime_guess::from_path(&file_path)
            .first()
            .map(|mime| mime.to_string());

        Ok((size, mime_type))
    }

    /// Delete an object
    pub async fn delete_object(&self, object_id: &str) -> TamsResult<()> {
        self.validate_object_id(object_id)?;
        
        let file_path = self.get_object_path(object_id);
        
        if file_path.exists() {
            fs::remove_file(&file_path).await?;
            tracing::info!("Deleted object {}", object_id);
        }

        Ok(())
    }

    /// List all objects (for cleanup and maintenance)
    pub async fn list_objects(&self) -> TamsResult<Vec<String>> {
        let mut objects = Vec::new();
        let mut entries = fs::read_dir(&self.config.base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                if let Some(file_name) = entry.file_name().to_str() {
                    objects.push(file_name.to_string());
                }
            }
        }

        Ok(objects)
    }

    /// Clean up temporary files older than the retention period
    pub async fn cleanup_temp_files(&self) -> TamsResult<u64> {
        let cutoff = Utc::now() - Duration::hours(self.config.temp_path.to_string_lossy().parse::<i64>().unwrap_or(24));
        let mut cleaned = 0u64;

        let mut entries = fs::read_dir(&self.config.temp_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if let Ok(modified) = metadata.modified() {
                let modified_utc: DateTime<Utc> = modified.into();
                if modified_utc < cutoff {
                    if let Err(e) = fs::remove_file(entry.path()).await {
                        tracing::warn!("Failed to remove temp file {:?}: {}", entry.path(), e);
                    } else {
                        cleaned += 1;
                    }
                }
            }
        }

        if cleaned > 0 {
            tracing::info!("Cleaned up {} temporary files", cleaned);
        }

        Ok(cleaned)
    }

    /// Generate a new object ID
    pub fn generate_object_id(&self) -> String {
        // Generate a UUID-based object ID with timestamp prefix for better locality
        let timestamp = Utc::now().timestamp();
        let uuid = Uuid::new_v4();
        format!("{:x}-{}", timestamp, uuid.simple())
    }

    /// Validate object ID format
    fn validate_object_id(&self, object_id: &str) -> TamsResult<()> {
        // Basic validation - object ID should be safe for filesystem
        if object_id.is_empty() || object_id.len() > 255 {
            return Err(TamsError::BadRequest("Invalid object ID length".to_string()));
        }

        // Check for dangerous characters
        if object_id.contains("..") || object_id.contains('/') || object_id.contains('\\') {
            return Err(TamsError::BadRequest("Invalid object ID format".to_string()));
        }

        Ok(())
    }

    /// Get the filesystem path for an object
    fn get_object_path(&self, object_id: &str) -> PathBuf {
        // Use a two-level directory structure for better performance
        // e.g., objects/ab/cd/abcd1234-5678-...
        let prefix = if object_id.len() >= 4 {
            format!("{}/{}", &object_id[0..2], &object_id[2..4])
        } else {
            "misc".to_string()
        };

        self.config.base_path
            .join(prefix)
            .join(object_id)
    }

    /// Get the filesystem path for a temporary file
    fn get_temp_path(&self, filename: &str) -> PathBuf {
        self.config.temp_path.join(filename)
    }

    /// Check if an object exists
    pub async fn object_exists(&self, object_id: &str) -> bool {
        let file_path = self.get_object_path(object_id);
        file_path.exists()
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> TamsResult<StorageStats> {
        let mut total_size = 0u64;
        let mut object_count = 0u64;

        fn visit_dir(dir: &Path, total_size: &mut u64, count: &mut u64) -> std::io::Result<()> {
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dir(&path, total_size, count)?;
                    } else {
                        *total_size += entry.metadata()?.len();
                        *count += 1;
                    }
                }
            }
            Ok(())
        }

        if let Err(e) = visit_dir(&self.config.base_path, &mut total_size, &mut object_count) {
            tracing::warn!("Error calculating storage stats: {}", e);
        }

        Ok(StorageStats {
            total_size_bytes: total_size,
            object_count,
            available_space_bytes: None, // TODO: Implement disk space checking
        })
    }
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_size_bytes: u64,
    pub object_count: u64,
    pub available_space_bytes: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (MediaStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        
        let config = MediaStorageConfig {
            base_path: temp_path.join("objects"),
            max_file_size: 1024 * 1024, // 1MB
            temp_path: temp_path.join("temp"),
        };

        let storage = MediaStorage::new(config, "http://localhost:8080".to_string()).unwrap();
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_store_and_retrieve_object() {
        let (storage, _temp_dir) = create_test_storage();
        
        let object_id = "test-object-123";
        let data = b"Hello, TAMS!".to_vec();

        // Store object
        storage.store_object(object_id, data.clone()).await.unwrap();

        // Retrieve object
        let retrieved_data = storage.get_object(object_id).await.unwrap();
        assert_eq!(data, retrieved_data);

        // Check metadata
        let (size, _mime_type) = storage.get_object_metadata(object_id).await.unwrap();
        assert_eq!(size, data.len() as u64);
    }

    #[tokio::test]
    async fn test_object_not_found() {
        let (storage, _temp_dir) = create_test_storage();
        
        let result = storage.get_object("nonexistent").await;
        assert!(matches!(result, Err(TamsError::ObjectNotFound { .. })));
    }

    #[tokio::test]
    async fn test_invalid_object_id() {
        let (storage, _temp_dir) = create_test_storage();
        
        let result = storage.store_object("../../../etc/passwd", b"hack".to_vec()).await;
        assert!(matches!(result, Err(TamsError::BadRequest(_))));
    }
} 