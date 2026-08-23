//! Bare import specifier classification: relative vs. self-reference vs. Node
//! builtin vs. a real package name that must be declared in `package.json`.

/// Node's built-in module names, both bare (`"fs"`) and `node:`-prefixed
/// (`"node:fs"`) forms. Neither form requires a `package.json` declaration.
const NODE_BUILTIN_MODULES: &[&str] = &[
    "assert",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "string_decoder",
    "sys",
    "timers",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

/// `true` for a relative specifier (`./foo`, `../foo`) — never a package.
pub(super) fn is_relative(specifier: &str) -> bool {
    specifier.starts_with('.')
}

/// `true` for a bare Node built-in module name, including a subpath import
/// (`"fs/promises"`). Never requires a `package.json` dependency declaration.
///
/// Only handles the bare form: `is_scheme_prefixed` already intercepts the
/// `node:`-prefixed form earlier in `emit_finding`, so a specifier reaching
/// this function has never had that prefix.
pub(super) fn is_node_builtin(name: &str) -> bool {
    let module = name.split('/').next().unwrap_or(name);
    NODE_BUILTIN_MODULES.contains(&module)
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

#[cfg(test)]
mod tests;
