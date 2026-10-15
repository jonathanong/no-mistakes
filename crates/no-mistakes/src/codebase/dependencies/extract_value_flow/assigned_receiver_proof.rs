use super::*;

impl ImportCollector {
    pub(super) fn record_rhs_acquisition(&mut self, value: &ValueExpression) {
        if self.receiver_only_bindings.is_none() {
            return;
        }
        match value {
            ValueExpression::Named { name, scope, .. } if !name.contains('.') => {
                if self.references_mutable_receiver(value) {
                    self.prior_member_mutations
                        .acquire_rhs_reference(scope.unwrap_or(0), name);
                }
            }
            ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
                for value in fields.values() {
                    self.record_rhs_acquisition(value);
                }
            }
            _ => {
                // Reuse the canonical descriptor projection, including static
                // member selectors, instead of scanning bindings or the AST.
                let mut references = Vec::new();
                nested::receiver_references(value, &[], &mut references);
                for ((scope, name), _) in references {
                    if self.has_mutable_receiver_binding(scope, &name) {
                        self.prior_member_mutations
                            .acquire_rhs_reference(scope, &name);
                    }
                }
            }
        }
    }

    /// Reuse canonical fresh-identity receipts that predate RHS fact demand.
    /// Missing or opaque receipts retain the conservative initial possibility.
    pub(super) fn assignment_rhs_parent_identities(
        &self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
        version: Option<mutable_origins::RootVersion>,
    ) -> Vec<Option<u32>> {
        if path.len() < 2
            || (!self.prior_member_mutations.has_assigned_origins()
                && self.assignment_rhs_frame.is_none())
        {
            return Vec::new();
        }
        let root = self
            .prior_member_mutations
            .history(key)
            .map(|(root, _)| root);
        let mut node = root;
        let mut identities = Vec::with_capacity(path.len() - 1);
        for end in 1..path.len() {
            let member = path[end - 1].as_deref();
            let identity = (|| {
                let root = root?;
                let parent = node?;
                let member = member?;
                let current = reads[end - 1];
                let (at, owner) = current?;
                let assigned = parent.assigned.as_ref()?;
                #[cfg(feature = "test-instrumentation")]
                self.prior_member_mutations.note_rhs_receipt_query();
                let capture = assigned.definitely_replaced(member, at, owner, None)?;
                let source_version = capture
                    .origin
                    .and_then(|read| self.mutable_receiver_version(key, read));
                if version.map(mutable_origins::RootVersion::identity)
                    != source_version.map(mutable_origins::RootVersion::identity)
                    || capture.reads.len() != end - 1
                    || capture.epoch != self.fresh_receiver_effects.epoch
                    || self
                        .fresh_receiver_effects
                        .called_between(capture.origin, current)
                    || !reference_identity::matching_ancestors(
                        root,
                        &path[..end - 1],
                        &reads[..end - 1],
                        capture,
                        Some(&self.fresh_receiver_effects),
                        None,
                    )
                {
                    return None;
                }
                Some(capture.completed_at)
            })();
            identities.push(identity);
            node = member.and_then(|member| node.and_then(|parent| parent.members.get(member)));
        }
        identities
    }

    pub(super) fn assignment_rhs_reference_intact(
        &mut self,
        key: &(usize, String),
        acquisition: (
            &[Option<String>],
            &[nested::ReceiverRead],
            usize,
            &identity_metadata::FreshCapture,
        ),
        info: mutation_projection::MutationInfo<'_>,
    ) -> bool {
        let (path, reads, prefix, capture) = acquisition;
        let epoch = std::mem::replace(&mut self.fresh_receiver_effects.epoch, info.metadata.epoch);
        let intact = {
            let saved = info.proof.and_then(|proof| proof.phases.as_ref());
            let phases = saved
                .or(self.fresh_receiver_effects.phases.as_ref())
                .map(|phases| phases.borrow());
            let owner = info
                .proof
                .map_or(self.destructuring_phase_owner, |proof| proof.owner);
            self.prior_member_mutations
                .history(key)
                .is_some_and(|(history, _)| {
                    intact(
                        history,
                        (&path[..prefix], &reads[..prefix]),
                        capture,
                        &self.fresh_receiver_effects,
                        phases
                            .as_deref()
                            .filter(|_| owner == Some(info.metadata.context))
                            .map(|phases| {
                                phases.cursor(
                                    &self.prior_member_mutations,
                                    key,
                                    info.metadata.context,
                                )
                            }),
                    )
                })
        };
        self.fresh_receiver_effects.epoch = epoch;
        intact
    }
}
fn intact(
    root: &mutation_history::ValueMutationHistory,
    coordinate: (&[Option<String>], &[nested::ReceiverRead]),
    capture: &identity_metadata::FreshCapture,
    effects: &fresh_receiver_effects::FreshReceiverEffects,
    phases: Option<active_reference_phases::PhaseCursor<'_>>,
) -> bool {
    let (path, reads) = coordinate;
    let Some((Some(member), parent)) = path.split_last() else {
        return false;
    };
    let Some(current @ Some((_, context))) = reads.last().copied() else {
        return false;
    };
    if capture.reads.len() != parent.len()
        || capture.epoch != effects.epoch
        || effects.called_between(capture.origin, current)
    {
        return false;
    }
    let mut node = root;
    let mut cursor = phases;
    for member in parent {
        let Some(member) = member else { return false };
        let Some(child) = node.members.get(member) else {
            return false;
        };
        node = child;
        cursor = cursor.map(|cursor| cursor.child(member));
    }
    if node.has_unassigned
        || cursor.is_some_and(|cursor| {
            !cursor.allowed() || cursor.changed_after_capture(member, capture, current)
        })
    {
        return false;
    }
    node.assigned.as_ref().is_some_and(|assigned| {
        !assigned.identity_changed(member, Some((capture.completed_at, context)), current)
    }) && reference_identity::matching_ancestors(
        root,
        parent,
        reads,
        capture,
        Some(effects),
        phases,
    )
}
