use regex::RegexSet;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct QueueWaitPatterns {
    patterns: Vec<String>,
    compiled: Arc<RegexSet>,
}

impl Default for QueueWaitPatterns {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            compiled: Arc::new(RegexSet::empty()),
        }
    }
}

impl PartialEq for QueueWaitPatterns {
    fn eq(&self, other: &Self) -> bool {
        self.patterns == other.patterns
    }
}

impl Eq for QueueWaitPatterns {}

impl QueueWaitPatterns {
    pub fn new(mut patterns: Vec<String>) -> Result<Self, String> {
        patterns.sort();
        patterns.dedup();
        let compiled = RegexSet::new(&patterns)
            .map_err(|error| format!("invalid queues.factories.enqueueAndWaitPatterns: {error}"))?;
        Ok(Self {
            patterns,
            compiled: Arc::new(compiled),
        })
    }

    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    pub(crate) fn matches(&self, name: &str) -> bool {
        self.compiled.is_match(name)
    }

    pub(super) fn merge(&mut self, other: &Self) -> Result<(), String> {
        if other.is_empty() || self == other {
            return Ok(());
        }
        *self = Self::new(
            self.patterns
                .iter()
                .chain(&other.patterns)
                .cloned()
                .collect(),
        )?;
        Ok(())
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "queues_patterns/tests.rs"]
mod tests;
