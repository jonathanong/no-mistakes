use super::index::Index;
use super::*;
use oxc_ast::ast::{Program, Statement};

pub(in super::super) struct Ready {
    helper: RowHelper,
    bindings: Vec<BoundedBinding>,
}

pub(in super::super) fn prepare(
    collector: &mut ImportCollector,
    program: &Program<'_>,
) -> Vec<Ready> {
    let module = program.body.iter().any(|statement| {
        matches!(
            statement,
            Statement::ImportDeclaration(_)
                | Statement::ExportAllDeclaration(_)
                | Statement::ExportNamedDeclaration(_)
                | Statement::ExportDefaultDeclaration(_)
        )
    });
    let (invalid, mut ready, unsafe_sources) = {
        let index = Index::new(collector);
        let invalid = index
            .lookups
            .iter()
            .filter(|(key, lookup)| !index.reads((key.0, key.1), &lookup.fields))
            .map(|(key, _)| (key.1.to_string(), key.0))
            .collect();
        let mut ready = Vec::new();
        let mut unsafe_sources = Vec::new();
        for candidate in super::candidates::collect(&index, module) {
            if let Some(bindings) = super::callers::bindings(&index, &candidate) {
                let mut fields: Vec<_> = candidate
                    .fields
                    .iter()
                    .map(|field| field.to_string())
                    .collect();
                fields.sort_unstable();
                ready.push(Ready {
                    helper: RowHelper {
                        owner: candidate.owner,
                        source: candidate.sources[0].clone(),
                        fields,
                    },
                    bindings,
                });
            } else {
                unsafe_sources.extend(candidate.sources.into_iter().cloned());
            }
        }
        (invalid, ready, unsafe_sources)
    };
    append(collector, invalid);
    append(collector, unsafe_sources);
    ready.sort_by_key(|ready| ready.helper.owner);
    ready
}

pub(in super::super) fn publish(collector: &mut ImportCollector, ready: Vec<Ready>) {
    let mutated: FxHashSet<_> = collector
        .value_flow
        .registry_mutations
        .iter()
        .map(|mutation| (mutation.scope, mutation.name.as_str()))
        .collect();
    for ready in ready {
        if mutated.contains(&(ready.helper.source.1, ready.helper.source.0.as_str())) {
            continue;
        }
        collector.value_flow.bounded_bindings.extend(ready.bindings);
        collector.value_flow.metadata_row_helpers.push(ready.helper);
    }
}

pub(in super::super) fn append(collector: &mut ImportCollector, keys: Vec<(String, usize)>) {
    let existing: FxHashSet<_> = collector
        .value_flow
        .registry_mutations
        .iter()
        .map(|mutation| (mutation.scope, mutation.name.as_str()))
        .collect();
    let mut keys: Vec<_> = keys
        .into_iter()
        .filter(|(name, scope)| !existing.contains(&(*scope, name.as_str())))
        .collect();
    keys.sort_unstable();
    keys.dedup();
    drop(existing);
    collector.value_flow.registry_mutations.extend(
        keys.into_iter()
            .map(|(name, scope)| RegistryMutation { name, scope }),
    );
}
