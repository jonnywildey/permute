use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime};
use sndfile::{OpenOptions, ReadOptions, SndFileIO};
use std::path::PathBuf;
use crate::permute_error::PermuteError;

const DEFAULT_MAX_MEMORY_BYTES: usize = 1024 * 1024 * 1024; // 1GB

#[derive(Clone)]
struct CachedAudio {
    samples: Arc<Vec<f64>>,
    size_bytes: usize,
    last_accessed: SystemTime,
}

pub struct AudioCache {
    cache: RwLock<HashMap<PathBuf, CachedAudio>>,
    current_memory: Arc<RwLock<usize>>,
    max_memory: usize,
}

impl Default for AudioCache {
    fn default() -> Self {
        Self::with_max_memory(DEFAULT_MAX_MEMORY_BYTES)
    }
}

impl AudioCache {
    pub fn with_max_memory(max_memory_bytes: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            current_memory: Arc::new(RwLock::new(0)),
            max_memory: max_memory_bytes,
        }
    }

    pub fn get_samples(&self, path: &str) -> Result<Arc<Vec<f64>>, PermuteError> {
        let path = PathBuf::from(path);
        
        // Try to get from cache first
        if let Some(cached) = self.cache.write().unwrap().get_mut(&path) {
            cached.last_accessed = SystemTime::now();
            return Ok(cached.samples.clone());
        }

        // Not in cache, need to load
        let mut snd = OpenOptions::ReadOnly(ReadOptions::Auto).from_path(&path)?;
        let samples: Vec<f64> = snd.read_all_to_vec()?;
        let size_bytes = samples.len() * std::mem::size_of::<f64>();

        // If this file would exceed max memory, don't cache it
        if size_bytes > self.max_memory {
            return Ok(Arc::new(samples));
        }

        // Make space if needed
        self.ensure_space(size_bytes);

        // Add to cache
        let cached = CachedAudio {
            samples: Arc::new(samples),
            size_bytes,
            last_accessed: SystemTime::now(),
        };

        let samples_arc = cached.samples.clone();
        
        let mut cache = self.cache.write().unwrap();
        cache.insert(path, cached);
        *self.current_memory.write().unwrap() += size_bytes;

        Ok(samples_arc)
    }

    fn ensure_space(&self, needed_bytes: usize) {
        let mut cache = self.cache.write().unwrap();
        let mut current_memory = self.current_memory.write().unwrap();

        while *current_memory + needed_bytes > self.max_memory && !cache.is_empty() {
            // Find least recently used entry
            let lru_path = cache
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(path, _)| path.clone());

            if let Some(path) = lru_path {
                if let Some(entry) = cache.remove(&path) {
                    *current_memory -= entry.size_bytes;
                }
            }
        }
    }

    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        *self.current_memory.write().unwrap() = 0;
    }
}

// Global singleton instance
lazy_static::lazy_static! {
    pub(crate) static ref AUDIO_CACHE: AudioCache = AudioCache::default();
} 