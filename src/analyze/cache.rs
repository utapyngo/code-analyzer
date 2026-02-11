// Copyright 2024 Block, Inc. (original code from https://github.com/block/goose)
// Copyright 2025 utapyngo (modifications)
// SPDX-License-Identifier: Apache-2.0

use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::lock_or_recover;
use super::types::{AnalysisMode, AnalysisResult};

#[derive(Clone)]
pub struct AnalysisCache {
    cache: Arc<Mutex<LruCache<CacheKey, Arc<AnalysisResult>>>>,
    #[allow(dead_code)]
    max_size: usize,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct CacheKey {
    path: PathBuf,
    modified: SystemTime,
    mode: AnalysisMode,
}

impl AnalysisCache {
    pub fn new(max_size: usize) -> Self {
        let size = NonZeroUsize::new(max_size).unwrap_or_else(|| {
            eprintln!(
                "Warning: Invalid cache size {}, using default 100",
                max_size
            );
            NonZeroUsize::new(100).unwrap()
        });

        Self {
            cache: Arc::new(Mutex::new(LruCache::new(size))),
            max_size,
        }
    }

    pub fn get(
        &self,
        path: &Path,
        modified: SystemTime,
        mode: &AnalysisMode,
    ) -> Option<AnalysisResult> {
        let mut cache = lock_or_recover(&self.cache, |c| c.clear());
        let key = CacheKey {
            path: path.to_path_buf(),
            modified,
            mode: *mode,
        };

        cache.get(&key).map(|result| (**result).clone())
    }

    pub fn put(
        &self,
        path: PathBuf,
        modified: SystemTime,
        mode: &AnalysisMode,
        result: AnalysisResult,
    ) {
        let mut cache = lock_or_recover(&self.cache, |c| c.clear());
        let key = CacheKey {
            path,
            modified,
            mode: *mode,
        };

        cache.put(key, Arc::new(result));
    }
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new(100)
    }
}
