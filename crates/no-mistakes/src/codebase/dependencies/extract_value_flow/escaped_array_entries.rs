use super::*;

pub(in super::super) fn array_elements(
    collector: &ImportCollector,
    value: &ValueExpression,
) -> Option<Vec<ValueExpression>> {
    Some(
        array_entries(collector, value)?
            .into_iter()
            .map(|(_, value)| value)
            .collect(),
    )
}

pub(in super::super) fn array_entries(
    collector: &ImportCollector,
    value: &ValueExpression,
) -> Option<Vec<(usize, ValueExpression)>> {
    let items = match value {
        ValueExpression::Array(items) => items.clone(),
        ValueExpression::Named { name, scope, .. } => {
            let indexes = collector
                .value_binding_indexes
                .get(&scope.unwrap_or(0))?
                .get(name)?;
            match &collector.value_flow.bindings[*indexes.last()?].value {
                ValueExpression::Array(items) => items.clone(),
                _ => return None,
            }
        }
        _ => return None,
    };
    let mut entries = Vec::with_capacity(items.len());
    for (key, value) in items.iter() {
        entries.push((key.parse::<usize>().ok()?, value.clone()));
    }
    entries.sort_by_key(|(slot, _)| *slot);
    Some(entries)
}

pub(in super::super) fn copied_member(
    collector: &mut ImportCollector,
    key: &(usize, String),
    path: &[Option<String>],
    at: u32,
) {
    let dotted = std::iter::once(key.1.as_str())
        .chain(path.iter().filter_map(|part| part.as_deref()))
        .collect::<Vec<_>>()
        .join(".");
    collector.invalidate_value_write_with_path(key.0, &dotted, at, Some(path), at);
}

pub(in super::super) fn assign_sources(
    collector: &ImportCollector,
    arguments: &[ValueExpression],
    raw: &[oxc_ast::ast::Argument<'_>],
) -> Vec<ValueExpression> {
    let mut sources = Vec::new();
    for (index, source) in arguments.iter().enumerate().skip(1) {
        if let Some(Argument::SpreadElement(spread)) = raw.get(index) {
            if let Some(entries) =
                array_entries(collector, &collector.value_expression(&spread.argument))
            {
                sources.extend(entries.into_iter().map(|(_, value)| value));
                continue;
            }
        }
        sources.push(source.clone());
    }
    sources
}

impl ImportCollector {
    pub(in super::super::super) fn object_assign_result(
        &self,
        call: &CallExpression<'_>,
    ) -> Option<ValueExpression> {
        if simple_callee_name(&call.callee).as_deref() != Some("Object.assign")
            || self.callee_binding_scope("Object").is_some()
        {
            return None;
        }
        let mut raw = call.arguments.iter();
        let Expression::ObjectExpression(target) =
            crate::codebase::ts_source::unwrap_ts_wrappers(raw.next()?.as_expression()?)
        else {
            return None;
        };
        let ValueExpression::Object(target_fields, null_prototype) = self.object_value(target)
        else {
            return None;
        };
        let mut fields = (*target_fields).clone();
        for source in raw {
            let Expression::ObjectExpression(object) =
                crate::codebase::ts_source::unwrap_ts_wrappers(source.as_expression()?)
            else {
                return None;
            };
            let ValueExpression::Object(source_fields, _) = self.object_value(object) else {
                return None;
            };
            fields.extend(
                source_fields
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone())),
            );
        }
        Some(ValueExpression::Object(
            std::sync::Arc::new(fields),
            null_prototype,
        ))
    }
}
