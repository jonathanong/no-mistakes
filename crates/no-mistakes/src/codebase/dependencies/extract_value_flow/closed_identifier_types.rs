use crate::codebase::dependencies::extract::CallableId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClosedStringRegistry {
    pub(crate) name: String,
    pub(crate) scope: usize,
    pub(crate) declared_at: u32,
    pub(crate) keys: Vec<String>,
    pub(crate) kind: RegistryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegistryKind {
    Object,
    Array,
    Map,
    Set,
    MetadataArray,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundedBinding {
    pub(crate) name: String,
    pub(crate) scope: usize,
    pub(crate) declared_at: u32,
    pub(crate) source: Option<(String, usize)>,
    pub(crate) members: Vec<String>,
    pub(crate) requires_guard: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowHelper {
    pub(crate) owner: CallableId,
    pub(crate) source: (String, usize),
    pub(crate) fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FiniteIdentifierGuard {
    pub(crate) context: Option<CallableId>,
    pub(crate) name: String,
    pub(crate) members: Vec<String>,
    pub(crate) from: u32,
    pub(crate) until: u32,
    pub(crate) kind: GuardKind,
    pub(crate) subject: Option<GuardSubject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuardKind {
    ClosedSet {
        collection: String,
        method: SetMethod,
    },
    Assertion {
        callee: String,
        argument: usize,
        call: u32,
    },
    Lookup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SetMethod {
    Has,
    Includes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegistryMutation {
    pub(crate) name: String,
    pub(crate) scope: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IdentifierWrite {
    pub(crate) scope: usize,
    pub(crate) name: String,
    pub(crate) members: Vec<Option<String>>,
    pub(crate) at: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuardSubject {
    Binding {
        scope: usize,
        declared_at: u32,
    },
    Parameter {
        scope: usize,
        owner: CallableId,
        index: usize,
    },
}
