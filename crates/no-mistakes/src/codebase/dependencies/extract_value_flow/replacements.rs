use super::*;

/// Sparse receiver execution and mutable callable-assignment evidence.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueReplacementFacts {
    /// Explicitly collected callable-only facts do not demand receiver resolution.
    /// Default remains conservative for externally prepared replacement evidence.
    pub callable_only: bool,
    pub callable_assignments: Vec<CallableAssignment>,
    pub callable_assignment_uses: FxHashMap<usize, FxHashSet<String>>,
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

impl ValueReplacementFacts {
    /// Default or any receiver execution evidence requires conservative resolution.
    pub fn requires_receiver_proof(&self) -> bool {
        !self.callable_only
            || !self.writes.is_empty()
            || !self.regions.is_empty()
            || !self.calls.is_empty()
            || self.needs_call_proofs
            || !self.unsafe_owners.is_empty()
    }
}
impl ValueFlowFacts {
    /// Literal callable assignments collected without allocating for ordinary files.
    pub fn callable_assignments(&self) -> &[CallableAssignment] {
        self.replacements
            .as_ref()
            .map_or(&[], |facts| &facts.callable_assignments)
    }
    /// Whether a lexical callable assignment escaped through a value use.
    pub fn callable_assignment_is_used(&self, scope: usize, name: &str) -> bool {
        self.replacements
            .as_ref()
            .and_then(|facts| facts.callable_assignment_uses.get(&scope))
            .is_some_and(|uses| uses.contains(name))
    }
    /// Whether prepared receiver evidence demands the canonical call graph.
    pub fn has_receiver_replacement_facts(&self) -> bool {
        self.replacements
            .as_ref()
            .is_some_and(|facts| facts.requires_receiver_proof())
    }
}

#[cfg(test)]
#[path = "replacement_capsule_tests.rs"]
mod capsule_tests;
