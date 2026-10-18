//! Production generation collection uses only the post-checkpoint graph.
pub(super) use super::super::reachable::collect_young as collect;

pub(super) fn pin_cutoff(cutoff: u64) -> u64 {
    cutoff
}
