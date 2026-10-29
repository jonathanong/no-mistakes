use super::*;

pub(super) fn uncertain_receivers(
    nodes: &[Node],
    values: &FxHashMap<usize, Value>,
) -> FxHashSet<usize> {
    let mut origins = values
        .values()
        .filter_map(|value| (value.kind == Kind::Receiver && !value.exact).then_some(value.origin))
        .collect::<FxHashSet<_>>();
    for node in nodes {
        if values.get(&node.id).is_some_and(|value| !value.exact) {
            if let Some(candidate) =
                evaluate(node, values).filter(|value| value.kind == Kind::Receiver)
            {
                origins.insert(candidate.origin);
            }
        }
    }
    origins
}

pub(super) fn eligible(profile: &EmbeddedSqlOptions, module: &TypeScriptModuleFacts) -> bool {
    !profile.import_specifier.is_empty()
        && profile.executor_names.iter().any(|name| name == "query")
        && module.imports.iter().any(|import| {
            import.specifier == profile.import_specifier
                && !import.type_only
                && import.bindings.iter().any(|binding| {
                    !binding.type_only && !profile.is_callable_export(&binding.imported)
                })
        })
}

pub(super) fn seeds(
    profile: &EmbeddedSqlOptions,
    module: &TypeScriptModuleFacts,
) -> FxHashMap<usize, Value> {
    let mut values = FxHashMap::default();
    for import in &module.imports {
        if import.specifier != profile.import_specifier || import.type_only {
            continue;
        }
        for binding in &import.bindings {
            if binding.type_only || profile.is_callable_export(&binding.imported) {
                continue;
            }
            values.insert(
                binding.binding_id,
                Value {
                    kind: Kind::Receiver,
                    origin: binding.binding_id,
                    exact: true,
                },
            );
        }
    }
    values
}
