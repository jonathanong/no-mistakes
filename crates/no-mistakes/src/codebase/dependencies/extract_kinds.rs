#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvocationKind {
    Call,
    Construct,
    Callback,
    /// Synthetic aggregate membership used by import reachability. This is
    /// not a JavaScript invocation and must never become a call edge.
    Membership,
    /// Class getter read, such as `C.value` or the read half of `C.value++`.
    Get,
    /// Class setter write, such as `C.value = next` or `C.value++`.
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallTargetIdentity {
    Global,
    ModuleExport,
    RepositoryFunction,
    Unknown,
}
