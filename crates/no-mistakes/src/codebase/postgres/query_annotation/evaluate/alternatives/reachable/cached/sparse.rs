use super::*;

#[cfg(not(test))]
#[macro_use]
mod hooks;
#[cfg(test)]
#[path = "sparse/tests/hooks.rs"]
#[macro_use]
mod hooks;

// Hash iteration visits capacity, including empty buckets. Retained oversized
// tables therefore use prefix lookups even when their current length is small.
fn visit_prefix<T>(entries: &FxHashMap<Environment, T>, base: usize, mut visit: impl FnMut(&T)) {
    if entries.is_empty() {
        return;
    }
    if entries.capacity() < base {
        for (&env, value) in entries {
            probe!();
            if env < base {
                visit(value);
            }
        }
    } else {
        for env in 0..base {
            probe!();
            if let Some(value) = entries.get(&env) {
                visit(value);
            }
        }
    }
}

pub(super) fn auxiliary_prefix(
    found: &mut Reachable,
    base: usize,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
) {
    legacy_dispatch!(found, base, mapped, captured);
    // Both tables remain live inputs: Scope identity does not track their writes.
    visit_prefix(captured, base, |origins| {
        // Prefix::seed already roots every environment below base. A warm
        // immutable summary can prove that none of these name edges add work.
        if origins.max_origin().is_some_and(|origin| origin >= base) {
            extend_origins!(found, origins);
        }
    });
    visit_prefix(mapped, base, |bindings| {
        found
            .pending_args
            .extend(bindings.iter().map(|(id, _)| *id));
    });
}

#[cfg(test)]
mod tests;
