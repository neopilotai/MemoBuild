use anyhow::{Context, Result};
use crate::remote_cache::RemoteCache;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub cache_key: String,
    pub created_at: i64,
    pub artifact_path: PathBuf,
    pub size: u64,
}

pub struct LocalCache {
    cache_dir: PathBuf,
    store: HashMap<String, CacheEntry>,
    index_path: PathBuf,
}

impl LocalCache {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;
        
        let index_path = cache_dir.join("index.json");
        let store = Self::load_index(&index_path)?;
        
        Ok(Self {
            cache_dir,
            store,
            index_path,
        })
    }

    fn get_cache_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".memobuild").join("cache"))
    }

    fn load_index(path: &Path) -> Result<HashMap<String, CacheEntry>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        
        let content = fs::read_to_string(path)?;
        let store: HashMap<String, CacheEntry> = serde_json::from_str(&content)
            .unwrap_or_default();
        
        Ok(store)
    }

    fn save_index(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.store)?;
        fs::write(&self.index_path, content)?;
        Ok(())
    }

    pub fn get_data(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(entry) = self.store.get(key) {
            let path = self.cache_dir.join(&entry.artifact_path);
            if path.exists() {
                return Ok(Some(fs::read(path)?));
            }
        }
        Ok(None)
    }

    pub fn put(&mut self, key: &str, data: &[u8]) -> Result<()> {
        let artifact_filename = format!("{}.bin", key);
        let artifact_path = PathBuf::from(&artifact_filename);
        let full_path = self.cache_dir.join(&artifact_path);
        
        fs::write(&full_path, data)?;
        
        let entry = CacheEntry {
            cache_key: key.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            artifact_path,
            size: data.len() as u64,
        };
        
        self.store.insert(key.to_string(), entry);
        self.save_index()?;
        
        Ok(())
    }

    pub fn exists(&self, key: &str) -> bool {
        self.store.contains_key(key)
    }
}

pub struct HybridCache<R: RemoteCache> {
    pub local: LocalCache,
    pub remote: Option<R>,
}

impl<R: RemoteCache> HybridCache<R> {
    pub fn new(remote: Option<R>) -> Result<Self> {
        Ok(Self {
            local: LocalCache::new()?,
            remote,
        })
    }

    pub fn get_artifact(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        // 1. Try local
        if let Some(data) = self.local.get_data(key)? {
            return Ok(Some(data));
        }

        // 2. Try remote
        if let Some(ref remote) = self.remote {
            if let Some(data) = remote.get(key)? {
                // Populate local cache
                self.local.put(key, &data)?;
                return Ok(Some(data));
            }
        }

        Ok(None)
    }

    pub fn put_artifact(&mut self, key: &str, data: &[u8]) -> Result<()> {
        // 1. Put local
        self.local.put(key, data)?;

        // 2. Put remote (async or fire-and-forget in real world, but let's keep it simple)
        if let Some(ref remote) = self.remote {
            remote.put(key, data)?;
        }

        Ok(())
    }
}
