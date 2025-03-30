use std::{collections::HashMap, sync::Mutex};
use lazy_static::lazy_static;

const MAX_CACHE_SIZE_BYTES: usize = 1024 * 1024 * 1024; // 1GB

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct CacheKey {
    file: String,
    window_size_bits: u64,  // Store as bits instead of f64
    target_length: usize,
    target_sample_rate: usize,
}

impl CacheKey {
    fn new(file: String, window_size_ms: f64, target_length: usize, target_sample_rate: usize) -> Self {
        Self {
            file,
            window_size_bits: window_size_ms.to_bits(),
            target_length,
            target_sample_rate,
        }
    }
}

#[derive(Debug)]
struct CacheEntry {
    rms_values: Vec<f64>,
    window_size_ms: f64,
    target_length: usize,
    target_sample_rate: usize,
    last_accessed: std::time::Instant,
}

impl CacheEntry {
    fn size_in_bytes(&self) -> usize {
        // Size of the RMS values (f64 array)
        let rms_size = self.rms_values.len() * std::mem::size_of::<f64>();
        // Size of the metadata (window_size_ms, target_length, target_sample_rate, last_accessed)
        let metadata_size = std::mem::size_of::<f64>() + 
                          std::mem::size_of::<usize>() * 2 + 
                          std::mem::size_of::<std::time::Instant>();
        rms_size + metadata_size
    }
}

#[derive(Debug)]
pub struct RmsCache {
    entries: HashMap<CacheKey, CacheEntry>,
    current_size: usize,
}

impl RmsCache {
    fn new() -> Self {
        RmsCache {
            entries: HashMap::new(),
            current_size: 0,
        }
    }

    fn get_rms(&mut self, key: &CacheKey) -> Option<Vec<f64>> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_accessed = std::time::Instant::now();
            Some(entry.rms_values.clone())
        } else {
            None
        }
    }

    fn insert_rms(
        &mut self,
        file: String,
        window_size_ms: f64,
        target_length: usize,
        target_sample_rate: usize,
        rms_values: Vec<f64>,
    ) {
        let key = CacheKey::new(file, window_size_ms, target_length, target_sample_rate);
        let entry = CacheEntry {
            rms_values,
            window_size_ms,
            target_length,
            target_sample_rate,
            last_accessed: std::time::Instant::now(),
        };

        let entry_size = entry.size_in_bytes();

        // If entry is too large for cache, don't cache it
        if entry_size > MAX_CACHE_SIZE_BYTES {
            return;
        }

        // Remove oldest entries until we have space
        while self.current_size + entry_size > MAX_CACHE_SIZE_BYTES {
            if let Some((oldest_key, oldest_entry)) = self.entries
                .iter()
                .map(|(k, v)| (k.clone(), v))
                .min_by_key(|(_, v)| v.last_accessed)
            {
                self.current_size -= oldest_entry.size_in_bytes();
                self.entries.remove(&oldest_key);
            } else {
                break;
            }
        }

        self.current_size += entry_size;
        self.entries.insert(key, entry);
    }
}

lazy_static! {
    pub static ref RMS_CACHE: Mutex<RmsCache> = Mutex::new(RmsCache::new());
}

pub fn get_cached_rms(
    file: &str,
    window_size_ms: f64,
    target_length: usize,
    target_sample_rate: usize,
) -> Option<Vec<f64>> {
    let key = CacheKey::new(file.to_string(), window_size_ms, target_length, target_sample_rate);
    RMS_CACHE.lock().unwrap().get_rms(&key)
}

pub fn cache_rms(
    file: String,
    window_size_ms: f64,
    target_length: usize,
    target_sample_rate: usize,
    rms_values: Vec<f64>,
) {
    RMS_CACHE.lock().unwrap().insert_rms(
        file,
        window_size_ms,
        target_length,
        target_sample_rate,
        rms_values,
    );
} 