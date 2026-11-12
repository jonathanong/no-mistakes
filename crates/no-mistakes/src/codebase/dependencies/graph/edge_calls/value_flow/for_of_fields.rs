use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn for_of_fields(
        &mut self,
        iteration: &ValueBinding,
        file: &CallableFileIndex,
        location: (u32, Option<CallableId>),
        member: &str,
        value: FlowValue,
    ) -> FlowValue {
        let (offset, _) = location;
        let FlowValue::Array {
            fields,
            file: source,
        } = &value
        else {
            return FlowValue::Unknown;
        };
        if fields.is_empty() || fields.len() > 32 {
            return FlowValue::Unknown;
        }
        let mut slots: Vec<_> = fields.keys().collect();
        slots.sort();
        let receiver_callables = self
            .facts
            .get_ts_facts(source)
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
            let selected_callable = matches!(row.get(member), Some(ValueExpression::Callable(_)));
            if !row.iter().all(|(key, value)| {
                key == member
                    || (selected_callable
                        && matches!(value, ValueExpression::Callable(id)
                        if receiver_callables.is_none_or(|owners| !owners.contains(id))))
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
            let element = self.member_at(value.clone(), slot, source, Some(location));
            if !matches!(element, FlowValue::Object { .. }) {
                return FlowValue::Unknown;
            }
            let target = self.member_at(element, member, source, Some(location));
            default_origins::merge(&mut combined, self.default_origins.take(), self.cache);
            values.push(target);
        }
        self.default_origins = combined;
        FlowValue::Choice(values)
    }
}
