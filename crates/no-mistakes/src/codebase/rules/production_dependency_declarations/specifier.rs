//! Bare import specifier classification: relative vs. self-reference vs. Node
//! builtin vs. a real package name that must be declared in `package.json`.

pub(super) use crate::codebase::ts_resolver::is_bare_node_builtin as is_node_builtin;

/// `true` for a relative specifier (`./foo`, `../foo`) — never a package.
pub(super) fn is_relative(specifier: &str) -> bool {
    specifier.starts_with('.')
}

/// `true` for a specifier using a non-npm URL/loader scheme (e.g. Vite's
/// `virtual:app-config`, a `data:` URI, webpack loader syntax), signaled by a
/// `:` in the specifier's first path segment. npm package names can never
/// contain `:`, so this can't misclassify a real package — it only stops a
/// scheme specifier from being parsed as one before it ever reaches
/// `package_name`.
pub(super) fn is_scheme_prefixed(specifier: &str) -> bool {
    specifier
        .split('/')
        .next()
        .is_some_and(|first| first.contains(':'))
}

/// Split a bare (non-relative, non-`#`) specifier into its package name,
/// discarding any subpath. Mirrors the parsing `workspaces::resolve_specifier`
/// uses internally, which is not reachable from this module.
pub(super) fn package_name(specifier: &str) -> Option<String> {
    if specifier.starts_with('.') || specifier.starts_with('/') || specifier.starts_with('#') {
        return None;
    }
    let mut parts = specifier.splitn(3, '/');
    let first = parts.next().unwrap_or("");
    if first.is_empty() {
        return None;
    }
    if let Some(scope_pkg) = first.starts_with('@').then(|| parts.next()).flatten() {
        return Some(format!("{first}/{scope_pkg}"));
    }
    Some(first.to_string())
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
