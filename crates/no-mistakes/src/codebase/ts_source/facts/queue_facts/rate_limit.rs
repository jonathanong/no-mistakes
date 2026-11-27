#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RateLimitSiteKind {
    /// Confirmed queue `RateLimitError` throw or worker `.rateLimit()` call.
    Confirmed,
    /// `RateLimitError` throw whose origin could not be traced locally or across barrels.
    Unresolved,
    /// Imported `throw new Foo()` / `throw Foo()` from a non-queue specifier.
    ImportedError {
        source: String,
        imported: String,
        is_rate_limit_name: bool,
    },
    /// `throw obj.RateLimitError()` where `obj` is a named import from a non-queue specifier.
    ImportedMemberObject { source: String, imported: String },
    /// `throw ns.RateLimitError()` where `ns` is `import * as ns from source`.
    NamespaceError { source: String },
    /// `throw ns.Worker.RateLimitError()` where `ns` is `import * as ns from source`.
    NamespaceWorkerError { source: String, member: String },
    /// `throw obj.Worker.RateLimitError()` where `obj` is a named import from a non-queue specifier.
    ImportedNamespaceWorkerError {
        source: String,
        imported: String,
        member: String,
    },
}

/// A `RateLimitError` throw or `.rateLimit()` call inside a processor.
#[derive(Debug, Clone)]
pub(crate) struct RateLimitSite {
    pub line: usize,
    pub worker_index: usize,
    pub kind: RateLimitSiteKind,
}

/// Top-level export classification used to trace `RateLimitError` through local barrels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RateLimitExport {
    FromNamed {
        source: String,
        imported: String,
    },
    FromNamespace {
        source: String,
    },
    FromMemberRateLimit {
        source: String,
        imported: Option<String>,
    },
    NonQueue,
    Opaque,
}
