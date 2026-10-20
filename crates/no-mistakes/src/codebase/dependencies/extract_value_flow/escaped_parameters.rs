use super::*;

impl ImportCollector {
    pub(in super::super::super) fn note_parameter_reassignment(
        &mut self,
        assignment: &AssignmentExpression<'_>,
    ) {
        if !self.collect_value_flow
            || !(assignment.operator.is_assign() || assignment.operator.is_logical())
        {
            return;
        }
        let Some(name) = super::super::mutable_receivers::mutable_root_name(&assignment.left)
        else {
            return;
        };
        let Some(scope) = self.callee_binding_scope(name) else {
            return;
        };
        let bare = self
            .value_parameter_indexes
            .get(&scope)
            .and_then(|names| names.get(name))
            .is_some_and(|indexes| {
                indexes.iter().any(|index| {
                    self.value_flow
                        .parameters
                        .get(*index)
                        .is_some_and(|parameter| parameter.projection.is_empty())
                })
            });
        if bare {
            self.receiver_escapes.reassigned.push((
                scope,
                name.to_string(),
                self.value_expression(&assignment.right),
            ));
        }
    }

    pub(in super::super::super) fn project_arguments_member(
        &mut self,
        offset: u32,
        path: Option<&[Option<String>]>,
        completed_at: u32,
    ) -> bool {
        let Some((head, tail)) = path.and_then(|path| path.split_first()) else {
            return false;
        };
        let Some(index) = head.as_deref().and_then(|text| text.parse().ok()) else {
            return false;
        };
        let Some(owner) = self.arguments_owner() else {
            return false;
        };
        let found = self
            .value_flow
            .parameters
            .iter()
            .find(|parameter| {
                parameter.owner == owner
                    && parameter.index == index
                    && parameter.projection.is_empty()
            })
            .map(|parameter| (parameter.scope, parameter.name.clone()));
        let (scope, name) =
            found.unwrap_or_else(|| self.synthesize_arguments_parameter(owner, index));
        let dotted = std::iter::once(name.as_str())
            .chain(tail.iter().filter_map(|part| part.as_deref()))
            .collect::<Vec<_>>()
            .join(".");
        self.invalidate_value_write_with_path(scope, &dotted, offset, Some(tail), completed_at);
        true
    }

    fn synthesize_arguments_parameter(
        &mut self,
        owner: CallableId,
        index: usize,
    ) -> (usize, String) {
        let scope = self.current_lexical_scope_id();
        let name = format!("arguments[{index}]");
        let order = self.value_flow.parameters.len();
        self.value_parameter_indexes
            .entry(scope)
            .or_default()
            .entry(name.clone())
            .or_default()
            .push(order);
        self.value_flow.parameters.push(ValueParameter {
            owner,
            index,
            name: name.clone(),
            scope,
            default: ValueExpression::Unknown,
            projection: Vec::new(),
            initialization_order: order,
            default_order: order,
            default_mutations: None,
            invalidated_at: None,
            mutation_contexts: fx_set(),
            type_reference: None,
        });
        (scope, name)
    }

    fn arguments_owner(&self) -> Option<CallableId> {
        self.function_id_stack
            .iter()
            .rev()
            .find(|(id, _)| !self.receiver_escapes.arrows.contains(id))
            .map(|(id, _)| *id)
    }

    pub(in super::super) fn project_arguments_alias(
        &mut self,
        scope: usize,
        name: &str,
        offset: u32,
        receiver: Option<&[Option<String>]>,
        completed_at: u32,
    ) -> bool {
        let base = name.split_once('.').map_or(name, |(base, _)| base);
        let Some(path) = receiver else {
            return false;
        };
        if !self
            .receiver_escapes
            .argument_aliases
            .contains(&(scope, base.to_string()))
        {
            return false;
        }
        self.project_arguments_member(offset, Some(path), completed_at)
    }
}
