#[cfg(feature = "test-instrumentation")]
pub(crate) fn last_summaries() -> usize {
    super::LAST_SUMMARIES.get()
}
