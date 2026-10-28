//! Keep unconditional exports separate from closed, plain dependency targets.
use super::Signature;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect(
    module: &TypeScriptModuleFacts,
    proven: &FxHashMap<(usize, u8), super::summary::Summary>,
) -> (BTreeMap<String, Signature>, BTreeSet<String>) {
    let roots = module
        .scopes
        .iter()
        .filter(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id)
        .collect::<FxHashSet<_>>();
    let mut locals: FxHashMap<&str, Option<usize>> = FxHashMap::default();
    for binding in module
        .bindings
        .iter()
        .filter(|b| b.runtime && !b.imported && roots.contains(&b.scope_id))
    {
        locals
            .entry(binding.name.as_str())
            .and_modify(|owner| *owner = None)
            .or_insert(Some(binding.id));
    }
    let mut seen = FxHashSet::default();
    let mut ambiguous = FxHashSet::default();
    let mut result = BTreeMap::new();
    let mut closed = BTreeSet::new();
    for export in module
        .exports
        .iter()
        .filter(|e| !e.type_only && e.specifier.is_none())
    {
        if !seen.insert(export.exported.as_str()) {
            result.remove(export.exported.as_str());
            closed.remove(export.exported.as_str());
            ambiguous.insert(export.exported.as_str());
        }
        if ambiguous.contains(export.exported.as_str()) {
            continue;
        }
        let Some(summary) = locals
            .get(export.local.as_str())
            .copied()
            .flatten()
            .and_then(|id| proven.get(&(id, 0)))
        else {
            continue;
        };
        if summary.required.is_some() {
            continue;
        }
        if summary.strict && summary.signature.plain() {
            closed.insert(export.exported.clone());
        }
        result.insert(export.exported.clone(), summary.signature);
    }
    (result, closed)
}
