use super::hooks;
use std::{path::Path, sync::Arc};

// Both builds execute this ordering; only the content component is instrumented.
#[inline]
pub(super) fn compare(a: &Arc<Path>, b: &Arc<Path>) -> bool {
    hooks::owner_identity!(a, b) || hooks::owner_component!(a.as_ref(), b.as_ref())
}
