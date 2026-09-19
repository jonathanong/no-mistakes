#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvocationKind {
    Call,
    Construct,
    Callback,
    /// Synthetic aggregate membership used by import reachability. This is
    /// not a JavaScript invocation and must never become a call edge.
    Membership,
    /// Getter read, such as `C.value`, `api.value`, or the read half of `C.value++`.
    Get,
    /// Setter write, such as `C.value = next`, `api.value = next`, or `C.value++`.
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallTargetIdentity {
    Global,
    ModuleExport,
    RepositoryFunction,
    Unknown,
}

/// Returns `true` for `.tsx` / `.jsx` files (which need the TSX grammar).
pub fn is_tsx_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("tsx" | "jsx")
    )
}

/// Returns `true` for any TypeScript/JavaScript source file we should index.
pub fn is_indexable(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("ts" | "mts" | "tsx" | "cts" | "js" | "mjs" | "jsx" | "cjs")
    )
}
