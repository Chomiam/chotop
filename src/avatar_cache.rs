use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Cache for downloaded avatar images
pub struct AvatarCache {
    cache_dir: PathBuf,
    cached: Arc<RwLock<HashMap<String, PathBuf>>>,
}

impl AvatarCache {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("discord-overlay")
            .join("avatars");

        // Create cache directory
        let _ = std::fs::create_dir_all(&cache_dir);

        Self {
            cache_dir,
            cached: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get avatar path, downloading if necessary
    pub async fn get_avatar(&self, user_id: &str, avatar_hash: &str) -> Option<PathBuf> {
        let cache_key = format!("{}_{}", user_id, avatar_hash);

        // Check if already cached
        {
            let cached = self.cached.read().await;
            if let Some(path) = cached.get(&cache_key) {
                if path.exists() {
                    return Some(path.clone());
                }
            }
        }

        // Download avatar
        let url = if avatar_hash.starts_with("http") {
            avatar_hash.to_string()
        } else {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png?size=64",
                user_id, avatar_hash
            )
        };

        let path = self.cache_dir.join(format!("{}.png", cache_key));

        match self.download_avatar(&url, &path).await {
            Ok(_) => {
                let mut cached = self.cached.write().await;
                cached.insert(cache_key, path.clone());
                Some(path)
            }
            Err(e) => {
                warn!("Failed to download avatar: {}", e);
                None
            }
        }
    }

    async fn download_avatar(&self, url: &str, path: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = reqwest::get(url).await?;
        let bytes = response.bytes().await?;
        tokio::fs::write(path, bytes).await?;
        info!("Downloaded avatar to {:?}", path);
        Ok(())
    }
}

impl Default for AvatarCache {
    fn default() -> Self {
        Self::new()
    }
}

// Add dirs crate dependency for cache directory
mod dirs {
    use std::path::PathBuf;

    pub fn cache_dir() -> Option<PathBuf> {
        std::env::var("XDG_CACHE_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".cache"))
            })
    }
}
