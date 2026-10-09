use super::super::EmbeddedSqlCall;

/// A relative import of a configured factory or type, unresolved during the fact parse.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RelativeScopedCandidate {
    pub(crate) specifier: String,
    pub(crate) imported_name: String,
    pub(crate) local_name: String,
    pub(crate) factory: bool,
    pub(crate) type_import: bool,
}

/// Source span that would bind `name` if one of `owners` resolves into the package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingRelativeSpan {
    pub(crate) owners: Vec<u32>,
    pub(crate) name: String,
    pub(crate) start: u32,
    pub(crate) end: u32,
}

/// Executor call recorded in source order, applied only if an owner is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingRelativeCall {
    pub(crate) seq: u32,
    pub(crate) owners: Vec<u32>,
    pub(crate) call: EmbeddedSqlCall,
}

/// Relative scoped imports from one file, cleared once projected.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct PendingRelativeScope {
    pub(crate) candidates: Vec<RelativeScopedCandidate>,
    pub(crate) spans: Vec<PendingRelativeSpan>,
    pub(crate) calls: Vec<PendingRelativeCall>,
    /// Sequence numbers parallel to confirmed `calls`, set only when provisional spans exist.
    pub(crate) confirmed_order: Vec<u32>,
    pub(crate) call_starts: std::collections::BTreeMap<u32, u32>,
}
