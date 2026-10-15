fn persistence_barrier(
    file: &CallableFileIndex,
    name: &str,
    property: &str,
    owner: Option<CallableId>,
) -> bool {
    let Some(history) = file
        .values
        .receiver_mutations
        .as_ref()
        .and_then(|histories| histories.get(&0))
        .and_then(|names| names.get(name))
    else {
        return false;
    };
    // A reassigned receiver or another writer invalidates persistence. The
    // property-qualified summary includes omitted aliases without replaying it.
    history
        .dynamic
        .as_ref()
        .is_some_and(|history| history.summary.is_some())
        || history
            .members
            .get(property)
            .and_then(|history| history.summary.as_ref())
            .is_some_and(|summary| summary.contexts.iter().any(|writer| *writer != owner))
        || history
            .assigned
            .as_ref()
            .and_then(|assigned| assigned.members.get(property))
            .is_some_and(|write| write.single_context() != Some(owner))
        || history
            .assigned
            .as_ref()
            .and_then(|assigned| assigned.dynamic.as_ref())
            .is_some_and(|write| write.single_context() != Some(owner))
}

fn stable_leaf(file: &CallableFileIndex, expression: &ValueExpression, depth: usize) -> bool {
    if depth > 32 {
        return false;
    }
    match expression {
        ValueExpression::Callable(_) => true,
        ValueExpression::Member(base, _) => stable_leaf(file, base, depth + 1),
        ValueExpression::Named { name, scope, .. } => {
            let base = name.split_once('.').map_or(name.as_str(), |(base, _)| base);
            let scope = scope.unwrap_or(0);
            if let Some(binding) = scoped_name(&file.values.bindings, scope, base) {
                stable_leaf(file, &binding.value, depth + 1)
            } else {
                scoped_name(&file.values.parameters, scope, base).is_none()
                    && (scope == 0 && file.imported.contains_key(base)
                        || file.resolve_local_callable_id(Some(scope), base).is_some())
            }
        }
        _ => false,
    }
}
fn unstable_rhs(
    flow: &mut ValueFlowResolution<'_, '_>,
    source: (&CallableFileIndex, &std::path::Path),
    expression: &ValueExpression,
    receivers: &FxHashMap<String, usize>,
    index: &PreparedReplacements,
    foreign: &super::writers::ForeignWriters<'_>,
    depth: usize,
) -> bool {
    if depth > 32 {
        return true;
    }
    let (file, path) = source;
    let (name, property, offset, owner) = match expression {
        ValueExpression::Named {
            name,
            scope,
            offset,
            context,
        } => match name.split_once('.') {
            Some((name, property)) => (name, property, *offset, *context),
            None => {
                return scoped_name(&file.values.bindings, scope.unwrap_or(0), name).is_some_and(
                    |binding| {
                        unstable_rhs(
                            flow,
                            source,
                            &binding.value,
                            receivers,
                            index,
                            foreign,
                            depth + 1,
                        )
                    },
                );
            }
        },
        ValueExpression::Member(base, property) => match base.as_ref() {
            ValueExpression::Named {
                name,
                offset,
                context,
                ..
            } => (name.as_str(), property.as_str(), *offset, *context),
            _ => return true,
        },
        _ => return false,
    };
    let Some(receiver) = receivers.get(name) else {
        return true;
    };
    let repeated_acquisition = owner.is_some()
        || index
            .region_at(owner, offset)
            .is_some_and(|id| index.facts.regions[id].may_repeat);
    // A method captured once outside repetition remains a snapshot. A live
    // source acquired in the repeated body cannot inherit a later RHS value.
    foreign.contains(*receiver, property, path)
        || repeated_acquisition
            && (index.reset(*receiver)
                || index.repeat_mutated(*receiver, property)
                || persistence_barrier(file, name, property, owner)
                || propagated_rhs_barrier(flow, file, name, property, offset, owner, path))
}

fn propagated_rhs_barrier(
    flow: &mut ValueFlowResolution<'_, '_>,
    file: &CallableFileIndex,
    name: &str,
    property: &str,
    offset: u32,
    owner: Option<CallableId>,
    path: &std::path::Path,
) -> bool {
    let Some(assigned) = file
        .values
        .receiver_mutations
        .as_ref()
        .and_then(|histories| histories.get(&0))
        .and_then(|names| names.get(name))
        .and_then(|history| history.assigned.as_ref())
        .and_then(|assigned| assigned.propagated.as_ref())
        .map(|propagated| &propagated.aggregate)
    else {
        return false;
    };
    let writes = [assigned.members.get(property), assigned.dynamic.as_ref()];
    if writes
        .iter()
        .flatten()
        .any(|write| write.single_context() != Some(owner) || write.offset <= offset)
    {
        return true;
    }
    // A later same-owner alias write cannot change an already copied module
    // method. A callable snapshot also needs one proven module execution;
    // otherwise another invocation can observe the previous alias replacement.
    writes.iter().any(Option::is_some)
        && owner.is_some_and(|owner| {
            !matches!(
                flow.source_execution_phase(path, owner),
                execution_quality::ExecutionPhase::Exact((_, None))
            )
        })
}
