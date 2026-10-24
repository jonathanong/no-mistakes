use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn caller_outputs(item: &SqlBoundItem) -> BTreeSet<String> {
    let SqlBoundItemKind::Query(query) = &item.kind else {
        return BTreeSet::new();
    };
    let mut named = BTreeMap::new();
    for (index, output) in query.outputs.iter().enumerate() {
        if let Some(name) = item
            .column_aliases
            .get(index)
            .cloned()
            .or_else(|| output.name.clone())
        {
            named
                .entry(name)
                .and_modify(|caller| *caller = false)
                .or_insert(output.caller_sized);
        }
    }
    named
        .into_iter()
        .filter_map(|(name, caller)| caller.then_some(name))
        .collect()
}

/// An aggregate row does not size its array: reuse the same prepared input with its
/// aggregate output cap removed, then let ArrayQuery prove rows and scalar elements.
pub(super) fn array_outputs(
    item: &SqlBoundItem,
) -> BTreeMap<String, crate::codebase::postgres::SqlBoundQuery> {
    let SqlBoundItemKind::Query(query) = &item.kind else {
        return BTreeMap::new();
    };
    if item.lateral || !item.lateral_reads.is_empty() || !item.lateral_qualified_reads.is_empty() {
        return BTreeMap::new();
    }
    let mut named = BTreeMap::new();
    let mut duplicates = BTreeSet::new();
    let mut seen = BTreeSet::new();
    for (index, output) in query.outputs.iter().enumerate() {
        let Some(name) = item.column_aliases.get(index).or(output.name.as_ref()) else {
            continue;
        };
        if !seen.insert(name.clone()) {
            duplicates.insert(name.clone());
        }
        let Some(column) = &output.array_aggregate_column else {
            continue;
        };
        let mut input = query.clone();
        input.capped = false;
        input.outputs = vec![crate::codebase::postgres::SqlBoundOutput {
            name: output.name.clone(),
            caller_sized: false,
            scalar: false,
            source_column: Some(column.clone()),
            array_aggregate_column: None,
        }];
        named.insert(name.clone(), input);
    }
    named.retain(|name, _| !duplicates.contains(name));
    named
}
