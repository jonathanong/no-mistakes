use super::*;

/// Registration maintains reverse readers incrementally. Compaction only
/// invalidates them when ownership entries disappear or environment IDs move.
pub(super) fn remap(
    captured: &mut FxHashMap<Environment, FxHashMap<String, Environment>>,
    preserved: Environment,
    indices: &FxHashMap<Environment, Environment>,
) -> bool {
    let changed = captured.iter().any(|(env, origins)| {
        (*env >= preserved && indices.get(env) != Some(env))
            || origins
                .values()
                .any(|origin| indices.get(origin).is_some_and(|new| new != origin))
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
            for origin in origins.values_mut() {
                if let Some(index) = indices.get(origin) {
                    *origin = *index;
                }
            }
            Some((key, origins))
        })
        .collect();
    true
}

#[cfg(test)]
mod tests;
