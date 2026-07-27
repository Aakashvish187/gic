use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Default maximum capacity for recent files list.
pub const DEFAULT_MAX_RECENT_FILES: usize = 10;

/// Individual entry in the Recent Files history list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentFileEntry {
    pub path: PathBuf,
    pub file_name: String,
    pub last_opened: SystemTime,
    pub size_bytes: u64,
}

impl RecentFileEntry {
    pub fn new<P: AsRef<Path>>(path: P, size_bytes: u64) -> Self {
        let p = path.as_ref().to_path_buf();
        let file_name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        Self {
            path: p,
            file_name,
            last_opened: SystemTime::now(),
            size_bytes,
        }
    }
}

/// Manager responsible for MRU (Most Recently Used) tracking, deduplication,
/// capacity eviction, and history pruning of recent files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentFilesManager {
    entries: Vec<RecentFileEntry>,
    max_capacity: usize,
}

impl Default for RecentFilesManager {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_RECENT_FILES)
    }
}

impl RecentFilesManager {
    /// Creates a new `RecentFilesManager` with specified max capacity.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_capacity: if max_capacity == 0 { 1 } else { max_capacity },
        }
    }

    /// Registers a file path in MRU order.
    /// If path already exists, moves entry to front and updates timestamp and size.
    pub fn add_path<P: AsRef<Path>>(&mut self, path: P, size_bytes: u64) {
        let target_path = path.as_ref();

        // Remove existing entry matching path if present
        self.entries.retain(|e| e.path != target_path);

        // Insert new entry at top (index 0)
        let new_entry = RecentFileEntry::new(target_path, size_bytes);
        self.entries.insert(0, new_entry);

        // Enforce max capacity eviction
        if self.entries.len() > self.max_capacity {
            self.entries.truncate(self.max_capacity);
        }
    }

    /// Removes a specific path from the recent list.
    pub fn remove_path<P: AsRef<Path>>(&mut self, path: P) -> bool {
        let target = path.as_ref();
        let len_before = self.entries.len();
        self.entries.retain(|e| e.path != target);
        self.entries.len() < len_before
    }

    /// Prunes entries whose target files no longer exist on disk.
    pub fn prune_nonexistent(&mut self) -> usize {
        let len_before = self.entries.len();
        self.entries.retain(|e| e.path.exists());
        len_before - self.entries.len()
    }

    /// Clears all recent file history.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns slice of current recent file entries (ordered from MRU to LRU).
    pub fn entries(&self) -> &[RecentFileEntry] {
        &self.entries
    }

    /// Returns current number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns current max capacity limit.
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }

    /// Sets max capacity limit and truncates if necessary.
    pub fn set_max_capacity(&mut self, max_capacity: usize) {
        self.max_capacity = if max_capacity == 0 { 1 } else { max_capacity };
        if self.entries.len() > self.max_capacity {
            self.entries.truncate(self.max_capacity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_recent_files_mru_order_and_capacity() {
        let mut manager = RecentFilesManager::new(3);

        manager.add_path("file1.txt", 100);
        manager.add_path("file2.txt", 200);
        manager.add_path("file3.txt", 300);

        assert_eq!(manager.len(), 3);
        assert_eq!(manager.entries()[0].file_name, "file3.txt");
        assert_eq!(manager.entries()[1].file_name, "file2.txt");
        assert_eq!(manager.entries()[2].file_name, "file1.txt");

        // Access file1.txt again -> should move to index 0
        manager.add_path("file1.txt", 105);
        assert_eq!(manager.len(), 3);
        assert_eq!(manager.entries()[0].file_name, "file1.txt");
        assert_eq!(manager.entries()[1].file_name, "file3.txt");
        assert_eq!(manager.entries()[2].file_name, "file2.txt");

        // Add 4th file -> file2.txt (oldest) evicted
        manager.add_path("file4.txt", 400);
        assert_eq!(manager.len(), 3);
        assert_eq!(manager.entries()[0].file_name, "file4.txt");
        assert_eq!(manager.entries()[1].file_name, "file1.txt");
        assert_eq!(manager.entries()[2].file_name, "file3.txt");
    }

    #[test]
    fn test_recent_files_remove_and_clear() {
        let mut manager = RecentFilesManager::new(5);
        manager.add_path("a.txt", 10);
        manager.add_path("b.txt", 20);

        assert!(manager.remove_path("a.txt"));
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.entries()[0].file_name, "b.txt");

        manager.clear();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_recent_files_prune_nonexistent() {
        let temp1 = NamedTempFile::new().unwrap();
        let path1 = temp1.path().to_path_buf();
        let path2 = PathBuf::from("non_existent_path_abc.txt");

        let mut manager = RecentFilesManager::new(5);
        manager.add_path(&path1, 50);
        manager.add_path(&path2, 50);

        assert_eq!(manager.len(), 2);
        let pruned_count = manager.prune_nonexistent();
        assert_eq!(pruned_count, 1);
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.entries()[0].path, path1);
    }

    #[test]
    fn test_recent_files_serialization() {
        let mut manager = RecentFilesManager::new(5);
        manager.add_path("sample.txt", 1024);

        let json = serde_json::to_string(&manager).unwrap();
        assert!(json.contains("sample.txt"));

        let deserialized: RecentFilesManager = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized.entries()[0].file_name, "sample.txt");
    }
}
