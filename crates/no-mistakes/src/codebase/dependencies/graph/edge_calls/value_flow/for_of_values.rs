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
        if let ValueExpression::Named {
            name,
            scope: Some(scope),
            ..
        } = &iteration.value
        {
            let Some(binding) = scoped_name(&file.values.bindings, *scope, name) else {
                return FlowValue::Unknown;
            };
            if file
                .values
                .for_of_escapes
                .as_ref()
                .and_then(|escaped| escaped.get(scope))
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
        let FlowValue::Array { fields, .. } = &value else {
            return FlowValue::Unknown;
        };
        if fields.is_empty() || fields.len() > 32 {
            return FlowValue::Unknown;
        }
        let mut slots: Vec<_> = fields.keys().collect();
        slots.sort();
        let receiver_callables = self
            .facts
            .get_ts_facts(path)
            .map(|facts| &facts.value_flow.for_of.receiver_callables);
        let imported_call = self.cache.ownership_callbacks
            && file
                .values
                .sites
                .get(&offset)
                .and_then(|sites| sites.get(&format!("{}.{member}", iteration.name)))
                .and_then(|key| file.values.calls.get(key))
                .is_some_and(|call| call.arguments.is_empty());
        let origins = self.default_origins.clone();
        let mut combined = None;
        let mut values = Vec::new();
        for slot in slots {
            let Some(ValueExpression::Object(row, _)) = fields.get(slot) else {
                return FlowValue::Unknown;
            };
            if !row.iter().all(|(key, value)| {
                key == member
                    || matches!(
                        value,
                        ValueExpression::Unknown
                            | ValueExpression::Undefined
                            | ValueExpression::Null
                    )
            }) {
                return FlowValue::Unknown;
            }
            let callable = matches!(row.get(member), Some(ValueExpression::Callable(id))
                if receiver_callables.is_none_or(|owners| !owners.contains(id)));
            let imported = imported_call
                && matches!(row.get(member), Some(ValueExpression::Named { name, scope: None | Some(0), .. }) if file.imported.contains_key(name));
            if !callable && !imported {
                return FlowValue::Unknown;
            }
            self.default_origins = origins.clone();
            let element = self.member_at(value.clone(), slot, path, Some((offset, context)));
            if !matches!(element, FlowValue::Object { .. }) {
                return FlowValue::Unknown;
            }
            let target = self.member_at(element, member, path, Some((offset, context)));
            default_origins::merge(&mut combined, self.default_origins.take(), self.cache);
            values.push(target);
        }
        self.default_origins = combined;
        FlowValue::Choice(values)
    }
}
