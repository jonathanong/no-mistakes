use super::*;

pub(in super::super) struct ReceiverOrigin {
    parameter: usize,
    origin: usize,
    path: Vec<String>,
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
    pub(super) fn invalidate_default_origins(
        &mut self,
        key: &(usize, String),
        path: &[String],
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
            if path.len() < receiver.path.len()
                || !path
                    .iter()
                    .zip(&receiver.path)
                    .all(|(actual, expected)| actual == expected || actual == "*")
            {
                continue;
            }
            let parameter = &mut self.value_flow.parameters[receiver.parameter];
            let mutations = std::sync::Arc::make_mut(
                parameter
                    .default_mutations
                    .get_or_insert_with(|| std::sync::Arc::new(fx_map())),
            );
            let suffix = &path[receiver.path.len()..];
            let paths = mutations.entry(receiver.origin).or_default();
            if let Some(mutation) = paths.get_mut(suffix) {
                mutation.contexts.insert(context);
                mutation.offset = mutation.offset.min(offset);
            } else {
                let mut contexts = fx_set();
                contexts.insert(context);
                paths.insert(suffix.to_vec(), ValueMutationInfo { offset, contexts });
            }
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
                Some(member.clone())
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
