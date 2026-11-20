//! Package specifiers and Vite's `parent > child` include form.

/// Bare Node built-in module names and their subpaths. `node:`-prefixed
/// forms are rejected before this list is consulted.
const NODE_BUILTINS: &[&str] = &[
    "assert",
    "assert/strict",
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
    "dns/promises",
    "domain",
    "events",
    "fs",
    "fs/promises",
    "http",
    "http2",
    "https",
    "inspector",
    "inspector/promises",
    "module",
    "net",
    "os",
    "path",
    "path/posix",
    "path/win32",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "readline/promises",
    "repl",
    "stream",
    "stream/consumers",
    "stream/promises",
    "stream/web",
    "string_decoder",
    "sys",
    "timers",
    "timers/promises",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "util/types",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

pub(super) fn is_node_builtin(specifier: &str) -> bool {
    let bare = specifier
        .trim()
        .strip_prefix("node:")
        .unwrap_or(specifier.trim());
    if NODE_BUILTINS.contains(&bare) {
        return true;
    }
    bare.split('/')
        .next()
        .is_some_and(|module| NODE_BUILTINS.contains(&module))
}

pub(super) fn package_name(specifier: &str) -> Option<String> {
    let specifier = specifier.trim();
    if specifier.is_empty()
        || specifier.starts_with(['.', '/'])
        || specifier.starts_with("node:")
        || specifier.contains([':', '\\'])
    {
        return None;
    }
    if let Some(rest) = specifier.strip_prefix('@') {
        let (scope, rest) = rest.split_once('/')?;
        let name = rest.split('/').next().unwrap_or("");
        if scope.is_empty() || name.is_empty() {
            return None;
        }
        return Some(format!("@{scope}/{name}"));
    }
    let name = specifier.split('/').next().unwrap_or("");
    (!name.is_empty()).then(|| name.to_string())
}

/// Coverage key for an include child or a runtime import. A Node built-in
/// stays the package name. Every other specifier keeps its subpath, because
/// Vite pre-bundles that exact entry.
pub(super) fn coverage_name(specifier: &str) -> Option<String> {
    let package = package_name(specifier)?;
    if is_node_builtin(specifier) {
        Some(package)
    } else {
        Some(specifier.trim().to_string())
    }
}

/// Consecutive `parent > child` steps. A direct include has none.
/// `a > b > c` yields `(a, b)` and then `(b, c)`.
pub(super) fn nested_pairs(entry: &str) -> Vec<(String, String)> {
    let parts: Vec<&str> = entry
        .split('>')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    parts
        .windows(2)
        .map(|pair| (pair[0].to_string(), pair[1].to_string()))
        .collect()
}
