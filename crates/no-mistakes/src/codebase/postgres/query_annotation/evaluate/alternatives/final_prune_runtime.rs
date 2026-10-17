// Compile-time production policy; the alternate sibling owns all test state.
pub(super) const fn original_prune() -> bool {
    false
}
pub(super) fn record_reuse() {}
