// Test instrumentation is erased from production builds.
#[inline(always)]
pub(super) fn record(_bindings: usize) {}
