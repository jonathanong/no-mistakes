//! Invocation deduplication belongs to one execution path, not its siblings.
use super::super::{callbacks::CallbackIdentity, Environment};
use crate::fx::{FxHashMap, FxHashSet};

type Seen = Option<FxHashSet<CallbackIdentity>>;

pub(super) fn accumulate(joined: &mut Seen, current: &Seen) {
    if let (Some(joined), Some(current)) = (joined, current) {
        joined.extend(current.iter().cloned());
    }
}

/// Seen keys do not keep frames alive. Only retained captured environments
/// survive, using exactly the compactor's identity mapping.
pub(super) fn remap(
    seen: &mut Seen,
    preserved: usize,
    indices: &FxHashMap<Environment, Environment>,
) {
    if let Some(seen) = seen {
        *seen = std::mem::take(seen)
            .into_iter()
            .filter_map(|mut key| {
                if key.2 >= preserved {
                    key.2 = *indices.get(&key.2)?;
                }
                Some(key)
            })
            .collect();
    }
}
