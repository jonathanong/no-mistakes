use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn for_of_member(
        &mut self,
        path: &std::path::Path,
        base: &ValueExpression,
        member: &str,
    ) -> Option<FlowValue> {
        let ValueExpression::Named {
            name,
            scope: Some(scope),
            offset,
            context,
        } = base
        else {
            return None;
        };
        if self.cache.ownership_callbacks
            && self
                .facts
                .get_ts_facts(path)
                .is_none_or(|facts| facts.value_flow.for_of.guarded_calls.contains(offset))
        {
            return Some(FlowValue::Unknown);
        }
        let file = self.indexes.file(self.facts, path)?;
        let iteration = file.values.for_of_calls.get(offset).filter(|binding| {
            binding.scope == *scope && binding.name == *name && binding.owner == *context
        })?;
        let value = self.for_of_value(path, iteration, &file, *offset, *context, member);
        if !self.cache.ownership_callbacks && matches!(value, FlowValue::Unknown) {
            return None;
        }
        Some(value)
    }

    fn for_of_value(
        &mut self,
        path: &std::path::Path,
        iteration: &ValueBinding,
        file: &CallableFileIndex,
        offset: u32,
        context: Option<CallableId>,
        member: &str,
    ) -> FlowValue {
        let mut imported = None;
        if let ValueExpression::Named { name, scope, .. } = &iteration.value {
            let scope = scope.unwrap_or(0);
            if self.cache.ownership_callbacks
                && self
                    .facts
                    .get_ts_facts(path)
                    .and_then(|facts| facts.value_flow.for_of.exported.get(&scope))
                    .is_some_and(|names| names.contains(name))
            {
                return FlowValue::Unknown;
            }
            let binding = scoped_name(&file.values.bindings, scope, name);
            if binding.is_none() && scope == 0 && !self.cache.ownership_callbacks {
                imported = self.imported_stage_array(path, file, name);
            }
            let Some(binding) = binding else {
                return imported.map_or(FlowValue::Unknown, |value| {
                    self.for_of_fields(iteration, file, (offset, context), member, value)
                });
            };
            if file
                .values
                .for_of_escapes
                .as_ref()
                .and_then(|escaped| escaped.get(&scope))
                .and_then(|names| names.get(name))
                .is_some_and(|contexts| {
                    binding.owner.is_none() || contexts.iter().any(|owner| *owner != binding.owner)
                })
            {
                return FlowValue::Unknown;
            }
            if !binding.immutable
                || binding.owner.is_some_and(|owner| Some(owner) != context)
                || !matches!(binding.value, ValueExpression::Array(_))
            {
                return FlowValue::Unknown;
            }
        } else if !matches!(iteration.value, ValueExpression::Array(_)) {
            return FlowValue::Unknown;
        }
        let value = self.binding_value(path, iteration, (offset, context), (offset, context), None);
        self.for_of_fields(iteration, file, (offset, context), member, value)
    }
}
