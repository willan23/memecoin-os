use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Content-addressed raw object store. Duplicates collapse to the same hash.
pub trait ObjectStore: Send + Sync {
    fn put(&self, bytes: &[u8], provider: &str) -> Result<StoredObject, String>;
    fn get(&self, hash: &str) -> Result<Vec<u8>, String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredObject {
    pub hash: String,
    pub bytes: usize,
    pub provider: String,
    pub path: String,
    pub duplicate: bool,
}

pub fn content_hash(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

pub struct LocalFsStore {
    root: PathBuf,
}

impl LocalFsStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn from_env() -> Self {
        let root = std::env::var("OBJECT_STORE_PATH").unwrap_or_else(|_| "data/raw".into());
        Self::new(root)
    }

    fn file_path(&self, hash: &str) -> PathBuf {
        self.root.join(&hash[..2]).join(hash)
    }
}

impl ObjectStore for LocalFsStore {
    fn put(&self, bytes: &[u8], provider: &str) -> Result<StoredObject, String> {
        let hash = content_hash(bytes);
        let path = self.file_path(&hash);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let duplicate = path.exists();
        if !duplicate {
            fs::write(&path, bytes).map_err(|e| e.to_string())?;
        }
        Ok(StoredObject {
            hash,
            bytes: bytes.len(),
            provider: provider.into(),
            path: path.to_string_lossy().into(),
            duplicate,
        })
    }

    fn get(&self, hash: &str) -> Result<Vec<u8>, String> {
        if hash.len() < 4 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid hash".into());
        }
        fs::read(self.file_path(hash)).map_err(|e| e.to_string())
    }
}

pub fn prune_older_than(root: &Path, max_age: std::time::Duration) -> Result<u32, String> {
    let cutoff = std::time::SystemTime::now() - max_age;
    let mut n = 0u32;
    if !root.exists() {
        return Ok(0);
    }
    for shard in fs::read_dir(root).map_err(|e| e.to_string())? {
        let shard = shard.map_err(|e| e.to_string())?;
        if !shard.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        for file in fs::read_dir(shard.path()).map_err(|e| e.to_string())? {
            let file = file.map_err(|e| e.to_string())?;
            let meta = file.metadata().map_err(|e| e.to_string())?;
            if meta.modified().map(|m| m < cutoff).unwrap_or(false) {
                let _ = fs::remove_file(file.path());
                n += 1;
            }
        }
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_dedupes_and_get_roundtrips() {
        let dir = std::env::temp_dir().join(format!("mcos-obj-{}", uuid::Uuid::new_v4()));
        let store = LocalFsStore::new(&dir);
        let a = store.put(b"hello-raw", "dexscreener").unwrap();
        let b = store.put(b"hello-raw", "dexscreener").unwrap();
        assert_eq!(a.hash, b.hash);
        assert!(b.duplicate);
        assert_eq!(store.get(&a.hash).unwrap(), b"hello-raw");
        let _ = fs::remove_dir_all(dir);
    }
}
