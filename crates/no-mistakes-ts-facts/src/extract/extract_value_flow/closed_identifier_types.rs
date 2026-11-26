use crate::extract::{CallableId, ValueExpression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedStringRegistry {
    pub name: String,
    pub scope: usize,
    pub declared_at: u32,
    pub keys: Vec<String>,
    pub kind: RegistryKind,
    pub const_tuple: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryKind {
    Object,
    Array,
    Map,
    Set,
    MetadataArray,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedBinding {
    pub name: String,
    pub scope: usize,
    pub declared_at: u32,
    pub source: Option<(String, usize)>,
    pub members: Vec<String>,
    pub requires_guard: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowHelper {
    pub owner: CallableId,
    pub source: (String, usize),
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiniteIdentifierGuard {
    pub context: Option<CallableId>,
    pub name: String,
    pub members: Vec<String>,
    pub from: u32,
    pub until: u32,
    pub kind: GuardKind,
    pub subject: Option<GuardSubject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardKind {
    ClosedSet {
        collection: String,
        method: SetMethod,
        scope: Option<usize>,
    },
    Assertion {
        callee: String,
        argument: usize,
        call: u32,
    },
    Lookup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetMethod {
    Has,
    Includes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryMutation {
    pub export_only: bool,
    pub name: String,
    pub scope: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierWrite {
    pub context: Option<CallableId>,
    pub scope: usize,
    pub name: String,
    pub members: Vec<Option<String>>,
    pub at: u32,
    pub assigned: Option<Box<IdentifierAssignment>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierAssignment {
    pub completed_at: u32,
    pub owner: Option<CallableId>,
    pub unconditional: bool,
    pub value: ValueExpression,
    pub initial: Option<Box<IdentifierInitializer>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierInitializer {
    pub declared_at: u32,
    pub owner: Option<CallableId>,
    pub value: ValueExpression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardSubject {
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

#[derive(Debug)]
pub struct GuardCall {
    pub index: usize,
    pub subject: Option<ValueExpression>,
    pub collection_scope: Option<usize>,
}
