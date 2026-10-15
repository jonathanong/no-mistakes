/// Ordered projection output keeps its common single receiver on the stack.
/// Overflow retains every additional branch in its original insertion order.
pub(super) struct ReferenceOutput<T> {
    first: Option<T>,
    overflow: Vec<T>,
}
impl<T> Default for ReferenceOutput<T> {
    fn default() -> Self {
        Self {
            first: None,
            overflow: Vec::new(),
        }
    }
}
impl<T> Extend<T> for ReferenceOutput<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, values: I) {
        let mut values = values.into_iter();
        if self.first.is_none() {
            self.first = values.next();
        }
        self.overflow.extend(values);
    }
}
impl<T> ReferenceOutput<T> {
    pub(super) fn into_iter(self) -> impl Iterator<Item = T> {
        self.first.into_iter().chain(self.overflow)
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn record_counts(&self, facts: &mut super::ValueFlowFacts) {
        facts.reference_output_heap_buffers += usize::from(self.overflow.capacity() > 0);
    }
}

#[cfg(test)]
#[path = "reference_output_tests.rs"]
mod tests;
