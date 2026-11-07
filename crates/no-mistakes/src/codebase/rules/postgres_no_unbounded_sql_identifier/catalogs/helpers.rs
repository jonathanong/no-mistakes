use super::*;

pub(super) fn escaped<'a>(
    index: &index::Index<'a>,
    graph: &DepGraph,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
) -> FxHashSet<Helper<'a>> {
    let mut escaped = FxHashSet::default();
    for (name, reads) in &index.reads {
        if reads
            .iter()
            .all(|(_, read)| read.kind == CatalogReadKind::Callee)
            && !index.mutated.contains(name)
        {
            continue;
        }
        if let Some(helper) = index.names.get(name) {
            escaped.insert(*helper);
        }
        if !index.imports.contains(name) {
            continue;
        }
        let selectors: Vec<_> = if index.namespaces.contains(name) {
            reads
                .iter()
                .filter_map(|(_, read)| read.path.first())
                .map(|member| unique_namespace_member(graph, facts, name.0, name.2, member))
                .collect()
        } else {
            vec![unique_export(
                graph,
                facts,
                &bindings.routes,
                name.0,
                name.2,
            )]
        };
        for unique in selectors.into_iter().flatten() {
            if let Some(helper) =
                index
                    .names
                    .get(&(unique.producer.as_path(), 0, unique.exported.as_str()))
            {
                escaped.insert(*helper);
            }
        }
    }
    escaped
}

pub(super) fn safe(index: &index::Index<'_>, helper: Helper<'_>, source: Source<'_>) -> bool {
    if index.bad_helpers.contains(&helper) {
        return false;
    }
    let Some(flow) = index.flows.get(helper.0) else {
        return false;
    };
    if flow.async_callables.contains(&helper.1) || flow.generator_callables.contains(&helper.1) {
        return false;
    }
    let Some(accessors) = index.accessors.get(&helper) else {
        return false;
    };
    let Some(returns) = index.returns.get(&helper) else {
        return false;
    };
    if accessors.len() != returns.len()
        || accessors.iter().any(|accessor| {
            accessor.scope != source.1
                || accessor.source != source.2
                || !index.parameters.get(&helper).is_some_and(|parameters| {
                    parameters.iter().any(|parameter| {
                        parameter.name == accessor.key
                            && parameter.projection.is_empty()
                            && parameter.invalidated_at.is_none()
                            && !index.mutated.contains(&(
                                helper.0,
                                parameter.scope,
                                parameter.name.as_str(),
                            ))
                    })
                })
        })
    {
        return false;
    }
    let Some(catalog) = index.sources.get(&source).filter(|catalog| catalog.record) else {
        return false;
    };
    let Some(calls) = index
        .incoming
        .get(&helper)
        .filter(|calls| !calls.is_empty())
    else {
        return false;
    };
    calls.iter().all(|call_key| {
        let Some(call) = index.calls.get(call_key) else {
            return false;
        };
        if call.bound || call.spread_at.is_some() {
            return false;
        }
        let ValueExpression::Named { name, scope, .. } = &call.callee else {
            return false;
        };
        if index
            .mutated
            .contains(&(call_key.0, scope.unwrap_or(0), name.as_str()))
        {
            return false;
        }
        let Some(results) = index.results.get(call_key) else {
            return false;
        };
        let [binding] = results.as_slice() else {
            return false;
        };
        binding.immutable
            && binding.invalidated_at.is_none()
            && !index
                .exports
                .contains(&(call_key.0, binding.scope, binding.name.as_str()))
            && !index
                .mutated
                .contains(&(call_key.0, binding.scope, binding.name.as_str()))
            && index
                .reads
                .get(&(call_key.0, binding.scope, binding.name.as_str()))
                .is_some_and(|reads| {
                    !reads.is_empty()
                        && reads.iter().all(|(at, read)| {
                            *at >= binding.declared_at
                                && read.kind == CatalogReadKind::Value
                                && catalog.fields.contains(&read.path)
                        })
                })
    })
}
