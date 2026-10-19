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

/// Numeric indexes. A non-index key refuses the whole array so a spread cannot
/// assign element 10 to parameter 2 by sorting the key as text.
pub(in super::super) fn array_entries(
    collector: &ImportCollector,
    value: &ValueExpression,
) -> Option<Vec<(usize, ValueExpression)>> {
    let items = match value {
        ValueExpression::Array(items) => items.clone(),
        ValueExpression::Named { name, scope, .. } => {
            let scope = scope.unwrap_or(0);
            let indexes = collector.value_binding_indexes.get(&scope)?.get(name)?;
            // Appended with the binding, so the last index addresses that value.
            let index = *indexes.last()?;
            match &collector.value_flow.bindings[index].value {
                ValueExpression::Array(items) => items.clone(),
                _ => return None,
            }
        }
        _ => return None,
    };
    let mut entries = Vec::with_capacity(items.len());
    for (key, value) in items.iter() {
        let slot = key.parse::<usize>().ok()?;
        entries.push((slot, value.clone()));
    }
    entries.sort_by_key(|(slot, _)| *slot);
    Some(entries)
}

pub(super) fn apply_escaped(
    values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>,
    parameters: &FxHashMap<usize, (usize, String)>,
    collector: &ImportCollector,
    offset: u32,
) {
    if let Some(extras) = collector.escaped_parameter_arguments.get(&offset) {
        for (index, extra) in extras {
            if let Some(key) = parameters.get(index) {
                values
                    .entry(key.clone())
                    .or_default()
                    .extend(extra.iter().cloned());
            }
        }
    }
    if let Some((start, extra)) = collector.escaped_open_arguments.get(&offset) {
        for (index, key) in parameters {
            if *index >= *start {
                values
                    .entry(key.clone())
                    .or_default()
                    .extend(extra.iter().cloned());
            }
        }
    }
}
