use super::*;

/// Full identity coverage preserves every valid reader frame. Out-of-range
/// origins keep their old meaning; only stale reader keys require the fallback.
pub(super) fn remap_after_compaction(
    captured: &mut CapturedBindings,
    old_scope_count: usize,
    preserved: Environment,
    indices: &FxHashMap<Environment, Environment>,
    environments_changed: bool,
    origin_probe: &mut impl FnMut(Environment),
    identity_probe: &mut impl FnMut(Environment),
) -> bool {
    // Empty ownership cannot move or disappear, regardless of the frame domain.
    if captured.is_empty() {
        return false;
    }
    let unchanged = !environments_changed
        && old_scope_count.checked_sub(preserved) == Some(indices.len())
        && indices.iter().all(|(old, new)| {
            identity_probe(*old);
            *old >= preserved && *old < old_scope_count && old == new
        })
        && captured.keys().all(|reader| *reader < old_scope_count);
    if unchanged {
        return false;
    }
    remap(captured, preserved, indices, origin_probe)
}

/// Registration maintains reverse readers incrementally. Compaction only
/// invalidates them when ownership entries disappear or environment IDs move.
pub(super) fn remap(
    captured: &mut CapturedBindings,
    preserved: Environment,
    indices: &FxHashMap<Environment, Environment>,
    origin_probe: &mut impl FnMut(Environment),
) -> bool {
    let changed = captured.iter().any(|(env, origins)| {
        (*env >= preserved && indices.get(env) != Some(env))
            || origins.values().any(|origin| {
                origin_probe(*origin);
                indices.get(origin).is_some_and(|new| new != origin)
            })
    });
    if !changed {
        return false;
    }
    *captured = std::mem::take(captured)
        .into_iter()
        .filter_map(|(env, mut origins)| {
            let key = if env < preserved {
                Some(env)
            } else {
                indices.get(&env).copied()
            }?;
            origins.remap(indices);
            Some((key, origins))
        })
        .collect();
    true
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod identity_tests;
