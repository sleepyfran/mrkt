use std::collections::HashMap;

use time::{Duration, OffsetDateTime};

/// Cache entry that holds data with an expiration time.
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: OffsetDateTime,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: OffsetDateTime::now_utc() + ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() > self.expires_at
    }
}

/// Simple in-memory cache with expiration for storing key-value pairs. The inner
/// storage is not thread-safe, so the entire cache needs to be wrapped in a
/// `Mutex` or similar if used in a multi-threaded context.
pub struct Cache<K, T>
where
    K: std::hash::Hash + Eq + Clone,
    T: Clone,
{
    /// Hash-map that maps keys to cache entries.
    store: HashMap<K, CacheEntry<T>>,
}

impl<K, T> Cache<K, T>
where
    K: std::hash::Hash + Eq + Clone,
    T: Clone,
{
    /// Creates a new empty cache.
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    /// Inserts data into the cache with a specified time-to-live (TTL).
    pub fn insert(&mut self, key: K, data: T, ttl: Duration) {
        let entry = CacheEntry::new(data, ttl);
        self.store.insert(key, entry);
    }

    /// Returns a reference to the cached data if it exists and is not expired.
    pub fn get_or_delete(&mut self, key: &K) -> Option<&T> {
        let is_expired = self
            .store
            .get(key)
            .map(|entry| entry.is_expired())
            .unwrap_or(false);

        if is_expired {
            self.store.remove(key);
            return None;
        }

        self.store.get(key).map(|entry| &entry.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_new_cache_is_empty() {
        let mut cache: Cache<String, i32> = Cache::new();
        assert_eq!(cache.get_or_delete(&"key".to_string()), None);
    }

    #[test]
    fn test_insert_and_get() {
        let mut cache = Cache::new();
        let key = "test_key".to_string();
        let value = 42;
        let ttl = Duration::seconds(10);

        cache.insert(key.clone(), value, ttl);

        assert_eq!(cache.get_or_delete(&key), Some(&42));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let mut cache: Cache<String, i32> = Cache::new();
        assert_eq!(cache.get_or_delete(&"nonexistent".to_string()), None);
    }

    #[test]
    fn test_insert_overwrites_existing_key() {
        let mut cache = Cache::new();
        let key = "test_key".to_string();
        let ttl = Duration::seconds(10);

        cache.insert(key.clone(), 42, ttl);
        cache.insert(key.clone(), 84, ttl);

        assert_eq!(cache.get_or_delete(&key), Some(&84));
    }

    #[test]
    fn test_expired_entry_returns_none() {
        let mut cache = Cache::new();
        let key = "test_key".to_string();
        let value = 42;
        let ttl = Duration::milliseconds(1); // Very short TTL

        cache.insert(key.clone(), value, ttl);

        // Wait for the entry to expire
        thread::sleep(std::time::Duration::from_millis(10));

        assert_eq!(cache.get_or_delete(&key), None);
    }

    #[test]
    fn test_expired_entry_is_removed_from_store() {
        let mut cache = Cache::new();
        let key = "test_key".to_string();
        let value = 42;
        let ttl = Duration::milliseconds(1);

        cache.insert(key.clone(), value, ttl);

        // Verify it's initially there
        assert!(cache.store.contains_key(&key));

        // Wait for expiration
        thread::sleep(std::time::Duration::from_millis(10));

        // Access the expired key
        cache.get_or_delete(&key);

        // Verify it's been removed from the store
        assert!(!cache.store.contains_key(&key));
    }

    #[test]
    fn test_multiple_keys_with_different_ttls() {
        let mut cache = Cache::new();
        let key1 = "key1".to_string();
        let key2 = "key2".to_string();

        // Insert one with short TTL, one with long TTL
        cache.insert(key1.clone(), 100, Duration::milliseconds(1));
        cache.insert(key2.clone(), 200, Duration::seconds(10));

        // Wait for first to expire
        thread::sleep(std::time::Duration::from_millis(10));

        assert_eq!(cache.get_or_delete(&key1), None);
        assert_eq!(cache.get_or_delete(&key2), Some(&200));
    }

    #[test]
    fn test_cache_with_string_values() {
        let mut cache = Cache::new();
        let key = 42;
        let value = "hello world".to_string();
        let ttl = Duration::seconds(10);

        cache.insert(key, value.clone(), ttl);

        assert_eq!(cache.get_or_delete(&key), Some(&value));
    }

    #[test]
    fn test_cache_entry_creation() {
        let data = "test data".to_string();
        let ttl = Duration::seconds(5);
        let entry = CacheEntry::new(data.clone(), ttl);

        assert_eq!(entry.data, data);
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_expiration() {
        let data = "test data".to_string();
        let ttl = Duration::milliseconds(1);
        let entry = CacheEntry::new(data, ttl);

        // Initially not expired
        assert!(!entry.is_expired());

        // Wait for expiration
        thread::sleep(std::time::Duration::from_millis(10));

        // Now should be expired
        assert!(entry.is_expired());
    }

    #[test]
    fn test_zero_ttl_expires_immediately() {
        let mut cache = Cache::new();
        let key = "test_key".to_string();
        let value = 42;
        let ttl = Duration::ZERO;

        cache.insert(key.clone(), value, ttl);

        // Should be expired immediately
        assert_eq!(cache.get_or_delete(&key), None);
    }

    #[test]
    fn test_negative_ttl_expires_immediately() {
        let mut cache = Cache::new();
        let key = "test_key".to_string();
        let value = 42;
        let ttl = Duration::seconds(-1);

        cache.insert(key.clone(), value, ttl);

        // Should be expired immediately
        assert_eq!(cache.get_or_delete(&key), None);
    }
}
