use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::ModuleSpan;
use crate::fx::{FxHashMap, FxHashSet};

pub(super) fn exports(ts: &TsFileFacts) -> FxHashSet<String> {
    let Some(module) = ts.module_bindings.as_deref() else {
        return FxHashSet::default();
    };
    let root = module
        .scopes
        .iter()
        .find(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id);
    let bindings: FxHashMap<_, _> = module
        .bindings
        .iter()
        .filter(|binding| Some(binding.scope_id) == root && !binding.imported)
        .map(|binding| (binding.name.as_str(), binding))
        .collect();
    let mut export_references: FxHashMap<&str, Vec<ModuleSpan>> = FxHashMap::default();
    for export in &module.exports {
        let Some(binding) = bindings.get(export.local.as_str()) else {
            continue;
        };
        if export.specifier.is_none() && !contains(export.span, binding.span) {
            export_references
                .entry(&export.local)
                .or_default()
                .push(export.span);
        }
    }
    let used: FxHashSet<_> = bindings
        .iter()
        .filter(|(name, binding)| {
            binding.references.iter().any(|reference| {
                (reference.runtime || reference.type_only)
                    && !export_references.get(**name).is_some_and(|spans| {
                        spans.iter().any(|span| contains(*span, reference.span))
                    })
            })
        })
        .map(|(name, _)| *name)
        .collect();
    module
        .exports
        .iter()
        .filter(|export| export.specifier.is_none() && used.contains(export.local.as_str()))
        .map(|export| export.exported.clone())
        .collect()
}

fn contains(outer: ModuleSpan, inner: ModuleSpan) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

#[cfg(test)]
mod tests {
    #[test]
    fn absent_prepared_bindings_do_not_exempt_exports() {
        assert!(super::exports(&Default::default()).is_empty());
    }
}
