use crate::integration_tests::types::ConfigProject;

#[cfg(test)]
mod tests;

/// Drop config-derived projects whose ownership region (`scope`) is a strict
/// ancestor of another matched config's region. When two unrelated configs both
/// glob the same spec — e.g. a broad `testDir` and a nested, more-specific one —
/// the spec belongs to the deepest `testDir`, so only the more-specific config's
/// target is emitted. Explicit-policy projects (`scope == None`) are intentional
/// and never dominated; equal or sibling scopes are kept (distinct configs may
/// legitimately share a spec).
pub(super) fn owning_projects<'a>(matched: &[&'a ConfigProject]) -> Vec<&'a ConfigProject> {
    matched
        .iter()
        .copied()
        .filter(|project| !is_dominated(project, matched))
        .collect()
}

fn is_dominated(project: &ConfigProject, matched: &[&ConfigProject]) -> bool {
    let Some(scope) = project.scope.as_deref() else {
        return false;
    };
    matched.iter().any(|other| {
        other.config != project.config
            && other
                .scope
                .as_deref()
                .is_some_and(|other_scope| is_strict_descendant(scope, other_scope))
    })
}

/// True when `descendant` names a directory strictly inside `ancestor`. The root
/// scope (`""` / `"."`) is an ancestor of every non-root scope.
fn is_strict_descendant(ancestor: &str, descendant: &str) -> bool {
    let ancestor = normalize_scope(ancestor);
    let descendant = normalize_scope(descendant);
    if ancestor == descendant {
        return false;
    }
    if ancestor.is_empty() {
        return !descendant.is_empty();
    }
    descendant
        .strip_prefix(ancestor)
        .is_some_and(|rest| rest.starts_with('/'))
}

fn normalize_scope(scope: &str) -> &str {
    if scope == "." {
        ""
    } else {
        scope
    }
}
