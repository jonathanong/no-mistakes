use super::*;

pub(in super::super) struct AssignmentRhsFrame {
    offset: u32,
    owner: Option<CallableId>,
    conditional: bool,
    value: ValueExpression,
    destinations: FxHashSet<Destination>,
}
#[derive(PartialEq, Eq, Hash)]
struct Destination {
    component: usize,
    root_identity: Option<mutable_origins::RootIdentity>,
    parent_identities: Vec<Option<u32>>,
    epoch: u64,
    phase: Option<(u32, u32)>,
    path: ValueMutationPath,
    reads: Vec<nested::ReceiverRead>,
}
impl ImportCollector {
    pub(in super::super) fn begin_assignment_rhs(
        &mut self,
        assignment: &AssignmentExpression<'_>,
    ) -> Option<Option<Box<AssignmentRhsFrame>>> {
        let (scope, name, interested, barrier) =
            self.assignment_member_value_interested(assignment)?;
        let value = self.value_expression(&assignment.right);
        let has_receiver = self.assigned_rhs_has_receiver(&value);
        let identity_only = barrier && !has_receiver;
        if !interested && !has_receiver && !identity_only {
            return None;
        }
        // Identity-only barriers never introduce a new receiver projection.
        let value = if identity_only {
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.reference_rhs_barriers += 1;
            }
            ValueExpression::Object(std::sync::Arc::new(fx_map()), false)
        } else {
            value
        };
        self.prior_member_mutations
            .require_reference_proof(scope, &name);
        // Start the canonical call/capture accounting before LHS/RHS effects.
        self.fresh_receiver_effects.capture(&[]);
        let frame = Box::new(AssignmentRhsFrame {
            offset: assignment.span.start,
            owner: self.current_function_id(),
            conditional: self.receiver_write_is_conditional || assignment.operator.is_logical(),
            destinations: fx_set(),
            value,
        });
        Some(self.assignment_rhs_frame.replace(frame))
    }
    pub(super) fn retain_assignment_rhs_destination(
        &mut self,
        key: &(usize, String),
        source: (
            &[Option<String>],
            &[nested::ReceiverRead],
            Option<nested::RootRead>,
        ),
        assigned: Option<&(Option<String>, u32)>,
        metadata: &mutation_metadata::MutationMetadata,
    ) -> bool {
        let Some(frame) = self.assignment_rhs_frame.as_ref() else {
            return false;
        };
        if frame.offset != metadata.offset || frame.owner != metadata.context {
            return false;
        }
        let (path, reads, root_read) = source;
        let version = root_read.and_then(|read| self.mutable_receiver_version(key, read));
        let root_identity = version.map(mutable_origins::RootVersion::identity);
        if path.iter().any(Option::is_none) {
            return false;
        }
        let component = self
            .prior_member_mutations
            .assigned_destination_component(key);
        let Some((Some(member), _)) = assigned else {
            return false;
        };
        let path = path
            .iter()
            .cloned()
            .chain(std::iter::once(Some(member.clone())))
            .collect::<Vec<_>>();
        let reads = reads
            .iter()
            .copied()
            .chain(std::iter::once(Some((metadata.offset, metadata.context))))
            .collect::<Vec<_>>();
        let parent_identities = self.assignment_rhs_parent_identities(key, &path, &reads, version);
        let frame = self
            .assignment_rhs_frame
            .as_mut()
            .expect("active RHS assignment");
        frame.destinations.insert(Destination {
            component,
            root_identity,
            parent_identities,
            epoch: metadata.epoch,
            phase: metadata.phase,
            path,
            reads,
        });
        true
    }
    pub(in super::super) fn finish_assignment_rhs(
        &mut self,
        assignment: &AssignmentExpression<'_>,
        previous: Option<Option<Box<AssignmentRhsFrame>>>,
    ) {
        let Some(previous) = previous else { return };
        let frame = self
            .assignment_rhs_frame
            .take()
            .expect("active RHS assignment");
        self.assignment_rhs_frame = previous;
        let value = frame.value;
        let complete =
            !frame.conditional && self.assignment_rhs_is_complete(&assignment.right, &value);
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.reference_rhs_destinations += frame.destinations.len();
        }
        let stable_alias = self.assignment_rhs_stable_alias(&value);
        for destination in frame.destinations {
            let capture = identity_metadata::FreshCapture {
                origin: Some((frame.offset, frame.owner)),
                reads: self
                    .fresh_receiver_effects
                    .capture(&destination.reads[..destination.reads.len() - 1]),
                epoch: destination.epoch,
                phase: destination.phase,
                completed_at: assignment.span.end,
            };
            self.prior_member_mutations.record_assigned_origin(
                destination.component,
                destination.path,
                value.clone(),
                assigned_receiver_origins::Store {
                    owner: frame.owner,
                    reads: &destination.reads,
                    completed: assignment.span.end,
                    complete,
                    stable_alias,
                    capture,
                    root_identity: destination.root_identity,
                    parent_identities: destination.parent_identities,
                },
            );
        }
    }
}
