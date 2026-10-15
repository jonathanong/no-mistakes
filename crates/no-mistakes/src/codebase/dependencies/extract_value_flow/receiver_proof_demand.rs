use super::*;

#[derive(Clone, Copy)]
pub(super) struct ProofDemand {
    pub(super) complete: Option<bool>,
    pub(super) acquired: bool,
}
impl ProofDemand {
    pub(super) fn join(
        left: Option<Self>,
        right: Option<Self>,
        left_history: bool,
        right_history: bool,
    ) -> Option<Self> {
        let complete = match (
            left.and_then(|proof| proof.complete),
            right.and_then(|proof| proof.complete),
        ) {
            (Some(left), Some(right)) => Some(left && right),
            (Some(proof), None) => Some(proof && !right_history),
            (None, Some(proof)) => Some(proof && !left_history),
            (None, None) => None,
        };
        (left.is_some() || right.is_some()).then_some(Self {
            complete,
            acquired: left.is_some_and(|proof| proof.acquired)
                || right.is_some_and(|proof| proof.acquired),
        })
    }
}
impl ReceiverHistories {
    pub(in super::super) fn require_reference_proof(&mut self, scope: usize, name: &str) {
        self.reference_proof_demand = true;
        let id = self.intern(scope, name);
        let root = self.root(id);
        let complete = !self.histories.contains_key(&root);
        // Acquiring an alias does not request full proof collection. Freeze
        // completeness at the original first full-demand timing only.
        self.proofs
            .entry(root)
            .or_insert(ProofDemand {
                complete: None,
                acquired: false,
            })
            .complete
            .get_or_insert(complete);
    }

    pub(in super::super) fn component_generation(&self) -> Option<usize> {
        self.generation.checked_add(1)
    }
    pub(in super::super) fn proof_component(&self, scope: usize, name: &str) -> Option<usize> {
        let root = self.root(*self.names.get(&scope)?.get(name)?);
        (self
            .proofs
            .get(&root)
            .is_some_and(|proof| proof.complete == Some(true)))
        .then_some(root)
    }
    pub(in super::super) fn has_reference_proof_demand(&self) -> bool {
        self.reference_proof_demand
    }

    pub(in super::super) fn requires_proof(&self, scope: usize, name: &str) -> bool {
        self.names
            .get(&scope)
            .and_then(|names| names.get(name))
            .is_some_and(|id| {
                self.proofs
                    .get(&self.root(*id))
                    .is_some_and(|proof| proof.complete == Some(true))
            })
    }

    pub(in super::super) fn acquire_rhs_reference(&mut self, scope: usize, name: &str) {
        let id = self.intern(scope, name);
        let root = self.root(id);
        self.proofs
            .entry(root)
            .or_insert(ProofDemand {
                complete: None,
                acquired: false,
            })
            .acquired = true;
    }
}
