use super::*;

/// Overflow traversals retain the original independent binding-cycle set.
#[derive(Default)]
pub(in super::super) struct ActiveKeys {
    first: Option<Key>,
    other: FxHashSet<Key>,
}
impl ActiveKeys {
    pub(super) fn contains(&self, key: &Key) -> bool {
        self.first.as_ref() == Some(key) || self.other.contains(key)
    }
    pub(super) fn insert(&mut self, key: Key) {
        if self.first.is_none() {
            self.first = Some(key);
        } else {
            self.other.insert(key);
        }
    }
    pub(super) fn remove(&mut self, key: &Key) -> bool {
        if self.first.as_ref() == Some(key) {
            self.first = None;
            true
        } else {
            self.other.remove(key)
        }
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn overflow_buffer(&self) -> usize {
        usize::from(self.other.capacity() > 0)
    }
}
