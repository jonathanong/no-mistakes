// Test builds select actual operation probes at the parent module boundary.
// Production has no instrumentation state, observer calls or counter effects.
#[inline(always)]
pub(super) fn capture() {}
#[inline(always)]
pub(super) fn restore() {}
#[inline(always)]
pub(super) fn fresh() {}
