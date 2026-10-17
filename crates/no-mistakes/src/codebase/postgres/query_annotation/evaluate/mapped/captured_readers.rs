use super::Environment;
use crate::fx::{FxHashMap, FxHashSet};

type Readers = FxHashMap<Environment, FxHashMap<String, FxHashSet<Environment>>>;

/// Transform the existing reverse relation with the same frame mapping as its
/// forward capture relation. Compaction maps surviving frame IDs injectively
/// into IDs at or above `preserved`; preserved readers cannot collide with them.
/// Names move without cloning. Temporary move lists and origin collisions can
/// allocate. Substantially sparse tables shrink after pruning; ordinary
/// compactions keep their storage instead of reallocating on every remap.
pub(super) fn remap<const COUNT_NAMES: bool>(
    readers: &mut Readers,
    preserved: Environment,
    indices: &FxHashMap<Environment, Environment>,
) -> usize {
    let mut retained_names = 0;
    // No preserved prefix or surviving frame means no reader can remain.
    // Drop tables directly instead of visiting each reader in a discarded graph.
    if preserved == 0 && indices.is_empty() {
        readers.clear();
        if substantially_sparse(0, readers.capacity()) {
            readers.shrink_to_fit();
        }
        return 0;
    }
    readers.retain(|_, names| {
        names.retain(|_, frames| {
            let mut moved = Vec::new();
            frames.retain(|old| {
                if *old < preserved {
                    return true;
                }
                match indices.get(old) {
                    Some(new) if new == old => true,
                    Some(new) => {
                        moved.push(*new);
                        false
                    }
                    None => false,
                }
            });
            frames.extend(moved);
            if substantially_sparse(frames.len(), frames.capacity()) {
                frames.shrink_to_fit();
            }
            !frames.is_empty()
        });
        if substantially_sparse(names.len(), names.capacity()) {
            names.shrink_to_fit();
        }
        if COUNT_NAMES {
            retained_names += names.len();
        }
        !names.is_empty()
    });
    // Remove every moving key before insertion: a destination can still be an
    // old key. Origins absent from the mapping stay unchanged, so an origin
    // collision still unions names/readers; reader-frame mappings are injective.
    let mut moved = Vec::new();
    readers.retain(|old, names| match indices.get(old) {
        Some(new) if new != old => {
            moved.push((*new, std::mem::take(names)));
            false
        }
        _ => true,
    });
    for (origin, names) in moved {
        match readers.entry(origin) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(names);
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                for (name, frames) in names {
                    match entry.get_mut().entry(name) {
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            entry.insert(frames);
                        }
                        std::collections::hash_map::Entry::Occupied(mut entry) => {
                            // Two counted name buckets become one after origin collision.
                            if COUNT_NAMES {
                                retained_names -= 1;
                            }
                            entry.get_mut().extend(frames);
                        }
                    }
                }
            }
        }
    }
    if substantially_sparse(readers.len(), readers.capacity()) {
        readers.shrink_to_fit();
    }
    retained_names
}

// Keep small tables and up to four times the live entries. Beyond that bound,
// a former large capture graph must not retain its entire bucket allocation.
fn substantially_sparse(len: usize, capacity: usize) -> bool {
    capacity > 16 && capacity > len.saturating_mul(4)
}

#[cfg(test)]
pub(super) mod tests;
