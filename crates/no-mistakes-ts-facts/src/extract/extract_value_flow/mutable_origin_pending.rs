/// The first pushed origin is the bottom of the same LIFO stack. Most
/// projections select one leaf and need no heap buffer at all.
#[derive(Default)]
pub(super) struct PendingOrigins {
    first: Option<(usize, bool)>,
    overflow: Vec<(usize, bool)>,
}
impl PendingOrigins {
    pub(super) fn extend(&mut self, entries: impl IntoIterator<Item = (usize, bool)>) {
        for entry in entries {
            if self.first.is_none() {
                debug_assert!(self.overflow.is_empty());
                self.first = Some(entry);
            } else {
                self.overflow.push(entry);
            }
        }
    }
    pub(super) fn pop(&mut self) -> Option<(usize, bool)> {
        self.overflow.pop().or_else(|| self.first.take())
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn spilled(&self) -> bool {
        self.overflow.capacity() > 0
    }
}
