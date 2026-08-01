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

/// `true` for a Node built-in module, in either the bare or `node:`-prefixed
/// form, including a subpath import (`"fs/promises"`, `"node:fs/promises"`).
/// These never require a `package.json` dependency declaration.
pub(super) fn is_node_builtin(name: &str) -> bool {
    let bare = name.strip_prefix("node:").unwrap_or(name);
    let module = bare.split('/').next().unwrap_or(bare);
    NODE_BUILTIN_MODULES.contains(&module)
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
