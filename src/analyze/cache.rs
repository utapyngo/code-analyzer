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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn sample_result() -> AnalysisResult {
        AnalysisResult::empty(10)
    }

    #[test]
    fn cache_put_and_get() {
        let cache = AnalysisCache::new(10);
        let path = PathBuf::from("/tmp/test.rs");
        let modified = SystemTime::now();
        let mode = AnalysisMode::Semantic;

        cache.put(path.clone(), modified, &mode, sample_result());
        let result = cache.get(&path, modified, &mode);
        assert!(result.is_some());
        assert_eq!(result.unwrap().line_count, 10);
    }

    #[test]
    fn cache_miss_on_different_time() {
        let cache = AnalysisCache::new(10);
        let path = PathBuf::from("/tmp/test.rs");
        let t1 = SystemTime::UNIX_EPOCH;
        let t2 = SystemTime::now();
        let mode = AnalysisMode::Semantic;

        cache.put(path.clone(), t1, &mode, sample_result());
        let result = cache.get(&path, t2, &mode);
        assert!(result.is_none());
    }

    #[test]
    fn cache_miss_on_different_mode() {
        let cache = AnalysisCache::new(10);
        let path = PathBuf::from("/tmp/test.rs");
        let modified = SystemTime::now();

        cache.put(
            path.clone(),
            modified,
            &AnalysisMode::Semantic,
            sample_result(),
        );
        let result = cache.get(&path, modified, &AnalysisMode::Structure);
        assert!(result.is_none());
    }

    #[test]
    fn cache_miss_on_different_path() {
        let cache = AnalysisCache::new(10);
        let modified = SystemTime::now();
        let mode = AnalysisMode::Semantic;

        cache.put(PathBuf::from("/a.rs"), modified, &mode, sample_result());
        let result = cache.get(Path::new("/b.rs"), modified, &mode);
        assert!(result.is_none());
    }

    #[test]
    fn cache_evicts_when_full() {
        let cache = AnalysisCache::new(2);
        let t = SystemTime::now();
        let mode = AnalysisMode::Semantic;

        cache.put(PathBuf::from("/a.rs"), t, &mode, sample_result());
        cache.put(PathBuf::from("/b.rs"), t, &mode, sample_result());
        cache.put(PathBuf::from("/c.rs"), t, &mode, sample_result());

        // /a.rs should have been evicted (LRU)
        assert!(cache.get(Path::new("/a.rs"), t, &mode).is_none());
        assert!(cache.get(Path::new("/c.rs"), t, &mode).is_some());
    }

    #[test]
    fn cache_default_works() {
        let cache = AnalysisCache::default();
        let t = SystemTime::now();
        cache.put(
            PathBuf::from("/x.rs"),
            t,
            &AnalysisMode::Semantic,
            sample_result(),
        );
        assert!(
            cache
                .get(Path::new("/x.rs"), t, &AnalysisMode::Semantic)
                .is_some()
        );
    }
}
