#[cfg(feature = "test-instrumentation")]
pub(crate) fn callback_summaries() -> usize {
    super::callback_aliases::tests::last_summaries()
}
