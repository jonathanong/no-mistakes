use super::*;

/// Sparse execution evidence retained only in files with receiver writes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueReplacementFacts {
    pub writes: Vec<ValueReplacementWrite>,
    pub regions: Vec<ValueRepeatRegion>,
    pub calls: FxHashMap<u32, ValueRepeatCall>,
    pub needs_call_proofs: bool,
    pub unsafe_owners: FxHashSet<CallableId>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueReplacementWrite {
    pub receiver: String,
    pub property: Option<String>,
    pub receiver_reset: bool,
    pub owner: Option<CallableId>,
    pub region: Option<usize>,
    pub completed_at: u32,
    pub value: ValueExpression,
    pub unconditional: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueRepeatRegion {
    pub start: u32,
    pub end: u32,
    pub owner: Option<CallableId>,
    pub repeated: bool,
    pub may_repeat: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueRepeatCall {
    pub owner: Option<CallableId>,
    pub region: Option<usize>,
    pub unconditional: bool,
}
