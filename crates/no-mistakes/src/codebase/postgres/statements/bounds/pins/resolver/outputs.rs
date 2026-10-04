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
