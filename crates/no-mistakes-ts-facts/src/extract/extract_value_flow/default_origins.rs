use super::*;

#[path = "default_mutations_finish.rs"]
mod finish;

pub(in super::super) struct ReceiverOrigin {
    parameter: usize,
    origin: usize,
    path: ValueMutationPath,
}
impl ImportCollector {
    pub(super) fn prepare_parameter_origins(&mut self) {
        if self.parameter_default_origins.is_some() {
            return;
        }
        let mut index = fx_map();
        for (slot, parameter) in self.value_flow.parameters.iter().enumerate() {
            index_parameter(&mut index, slot, parameter);
        }
        self.parameter_default_origins = Some(index);
    }
    pub(super) fn record_parameter_origins(&mut self, slot: usize) {
        if let Some(index) = &mut self.parameter_default_origins {
            index_parameter(index, slot, &self.value_flow.parameters[slot]);
        }
    }
    pub(super) fn remember_member_mutation(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
        assigned: Option<(Option<String>, u32)>,
        tracking: mutation_metadata::Tracking<'_>,
    ) {
        if self.receiver_escapes.projecting_possible {
            self.remember_possible_escape(key, path, assigned, &tracking);
            return;
        }
        if self.remember_module_provider_mutation(key, path, assigned.clone(), &tracking) {
            return;
        }
        let mutation_metadata::Tracking {
            metadata,
            propagated,
            root_read,
        } = tracking;
        let rhs = self.retain_assignment_rhs_destination(
            key,
            (path, reads, root_read),
            assigned.as_ref(),
            metadata,
        );
        let offset = metadata.offset;
        let context = metadata.context;
        let capture_allowed = !self.fresh_receiver_effects.rebound(key.0, &key.1);
        let captured = assigned.as_ref().and_then(|(_, completed_at)| {
            (capture_allowed
                && metadata.fresh
                && (metadata.unconditional || metadata.region.is_some())
                && self.prior_member_mutations.requires_proof(key.0, &key.1))
            .then(|| identity_metadata::FreshCapture {
                origin: Some((offset, context)),
                epoch: metadata.epoch,
                reads: self.fresh_receiver_effects.capture(reads),
                phase: metadata.phase,
                completed_at: *completed_at,
            })
        });
        let (history, proof) = self.prior_member_mutations.history_mut(key);
        // Projected default reads retain guards for writes without an exact leaf.
        history.has_unassigned |= assigned.is_none();
        if let Some((assigned, completed_at)) = assigned {
            history.record_assignment(
                path,
                assigned,
                mutation_assignments::AssignmentTiming {
                    started_at: offset,
                    context,
                    completed: mutation_history::ValueAssignedMutation {
                        offset: completed_at,
                        writer: mutation_history::ValueAssignedWriter::single(context),
                        unconditional_offset: metadata
                            .unconditional
                            .then_some(completed_at)
                            .and_then(std::num::NonZeroU32::new),
                    },
                },
                propagated,
                (proof || rhs).then_some(metadata.fresh),
                (metadata.region, captured),
            );
        } else {
            history.record(path, offset, context);
        }
    }
}

fn index_parameter(index: &mut DefaultOriginIndex, slot: usize, parameter: &ValueParameter) {
    index_origin(index, slot, 0, &parameter.default, &parameter.projection);
    for (step, projection) in parameter.projection.iter().enumerate() {
        if let ValueProjection::Default(value, _) = projection {
            index_origin(
                index,
                slot,
                step + 1,
                value,
                &parameter.projection[step + 1..],
            );
        }
    }
}
fn index_origin(
    index: &mut DefaultOriginIndex,
    parameter: usize,
    origin: usize,
    value: &ValueExpression,
    projection: &[ValueProjection],
) {
    let path = projection
        .iter()
        .filter_map(|step| match step {
            ValueProjection::Member(member) | ValueProjection::Element(member) => {
                Some(Some(member.clone()))
            }
            ValueProjection::Default(_, _) => None,
        })
        .collect::<Vec<_>>();
    let mut references = Vec::new();
    nested::receiver_references(value, &path, &mut references);
    for ((scope, name), path) in references {
        index
            .entry(scope)
            .or_default()
            .entry(name)
            .or_default()
            .push(ReceiverOrigin {
                parameter,
                origin,
                path,
            });
    }
}
