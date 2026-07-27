use std::collections::HashSet;

/// Tags for categorizing and filtering rules.
#[derive(Debug, Clone, Default)]
pub struct RuleTags {
    tags: HashSet<String>,
}

impl RuleTags {
    /// Creates a new empty set of tags.
    pub fn new() -> Self {
        Self {
            tags: HashSet::new(),
        }
    }

    /// Adds a tag to the collection.
    pub fn add<S: Into<String>>(mut self, tag: S) -> Self {
        self.tags.insert(tag.into());
        self
    }

    /// Checks if a specific tag exists.
    pub fn has(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    /// Returns an iterator over all tags.
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.tags.iter()
    }
}
