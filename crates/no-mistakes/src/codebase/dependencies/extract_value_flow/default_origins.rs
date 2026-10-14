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
            let mut added = fx_map();
            index_parameter(&mut added, slot, &self.value_flow.parameters[slot]);
            for (scope, names) in added {
                for (name, receivers) in names {
                    if let Some(history) = self
                        .prior_member_mutations
                        .get(&scope)
                        .and_then(|names| names.get(name.as_str()))
                    {
                        for receiver in &receivers {
                            for (path, mutation) in history {
                                attach_mutation(
                                    &mut self.value_flow.parameters[slot],
                                    receiver,
                                    path,
                                    mutation.offset,
                                    mutation.contexts.iter().copied(),
                                );
                            }
                        }
                    }
                    index
                        .entry(scope)
                        .or_default()
                        .entry(name)
                        .or_default()
                        .extend(receivers);
                }
            }
        }
    }
    pub(super) fn remember_member_mutation(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        offset: u32,
    ) {
        let context = self.current_function_id();
        let paths = self
            .prior_member_mutations
            .entry(key.0)
            .or_default()
            .entry(key.1.clone())
            .or_default();
        let mutation = paths
            .entry(path.to_vec())
            .or_insert_with(|| ValueMutationInfo {
                offset,
                contexts: fx_set(),
            });
        mutation.offset = mutation.offset.min(offset);
        mutation.contexts.insert(context);
    }
    pub(super) fn invalidate_default_origins(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        offset: u32,
    ) {
        let context = self.current_function_id();
        for receiver in self
            .parameter_default_origins
            .as_ref()
            .and_then(|index| index.get(&key.0))
            .and_then(|names| names.get(key.1.as_str()))
            .into_iter()
            .flatten()
        {
            attach_mutation(
                &mut self.value_flow.parameters[receiver.parameter],
                receiver,
                path,
                offset,
                std::iter::once(context),
            );
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

fn attach_mutation(
    parameter: &mut ValueParameter,
    receiver: &ReceiverOrigin,
    path: &[Option<String>],
    offset: u32,
    contexts: impl Iterator<Item = Option<CallableId>>,
) {
    if path.len() < receiver.path.len()
        || !path
            .iter()
            .zip(&receiver.path)
            .all(|(actual, expected)| actual.is_none() || actual == expected)
    {
        return;
    }
    let mutations = std::sync::Arc::make_mut(
        parameter
            .default_mutations
            .get_or_insert_with(|| std::sync::Arc::new(fx_map())),
    );
    let paths = mutations.entry(receiver.origin).or_default();
    let mutation = paths
        .entry(path[receiver.path.len()..].to_vec())
        .or_insert_with(|| ValueMutationInfo {
            offset,
            contexts: fx_set(),
        });
    mutation.offset = mutation.offset.min(offset);
    mutation.contexts.extend(contexts);
}
