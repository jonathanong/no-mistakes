use super::*;

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
        assigned: Option<(Option<String>, u32)>,
        offset: u32,
        propagated: bool,
    ) {
        let context = self.current_function_id();
        let unconditional =
            !self.receiver_write_is_conditional && self.receiver_replacement_is_fresh;
        let (history, proof) = self.prior_member_mutations.history_mut(key);
        history.record(path, offset, context);
        if let Some((assigned, completed_at)) = assigned {
            let write = mutation_history::ValueAssignedMutation {
                offset: completed_at,
                writer: mutation_history::ValueAssignedWriter::single(context),
                unconditional_offset: unconditional
                    .then(|| std::num::NonZeroU32::new(completed_at))
                    .flatten(),
            };
            history.record_assignment(
                path,
                assigned,
                write,
                propagated,
                proof.then_some(self.receiver_replacement_is_fresh),
            );
        }
    }
    pub(in super::super) fn finish_default_mutations(&mut self) {
        let Some(index) = self.parameter_default_origins.take() else {
            return;
        };
        let mut histories = std::mem::take(&mut self.prior_member_mutations);
        let components = histories.finish();
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.reference_component_links = histories.links;
            self.value_flow.reference_component_root_steps = histories.root_steps.get();
        }
        let mut retained = fx_map();
        for (component, mut history) in components {
            history.finish_early_assignments();
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.early_assignment_normalization_visits +=
                    history.early_phase_entries();
            }
            let history = std::sync::Arc::new(history);
            for (scope, name) in component {
                retained
                    .entry(scope)
                    .or_insert_with(fx_map)
                    .insert(name.clone(), std::sync::Arc::clone(&history));
                // A late alias captures the already mutated receiver, including
                // a synthetic default export. Apply the shared cutoff once.
                if let Some(summary) = &history.summary {
                    for slot in self
                        .value_binding_indexes
                        .get(&scope)
                        .and_then(|names| names.get(name.as_str()))
                        .into_iter()
                        .flatten()
                    {
                        let binding = &mut self.value_flow.bindings[*slot];
                        binding.invalidated_at = Some(
                            binding
                                .invalidated_at
                                .map_or(summary.offset, |offset| offset.min(summary.offset)),
                        );
                        binding
                            .mutation_contexts
                            .extend(summary.contexts.iter().copied());
                    }
                }
                for receiver in index
                    .get(&scope)
                    .and_then(|names| names.get(name.as_str()))
                    .into_iter()
                    .flatten()
                {
                    let parameter = &mut self.value_flow.parameters[receiver.parameter];
                    let mutations = std::sync::Arc::make_mut(
                        parameter
                            .default_mutations
                            .get_or_insert_with(|| std::sync::Arc::new(fx_map())),
                    );
                    mutations.entry(receiver.origin).or_default().push(
                        ValueDefaultMutationSource {
                            path: receiver.path.clone(),
                            history: std::sync::Arc::clone(&history),
                        },
                    );
                }
            }
        }
        self.value_flow.receiver_mutations = Some(std::sync::Arc::new(retained));
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
