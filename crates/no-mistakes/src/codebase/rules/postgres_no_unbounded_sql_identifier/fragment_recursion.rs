use super::*;
use crate::codebase::dependencies::extract::{
    CallableId, InvocationKind, ValueBinding, ValueCall, ValueFlowFacts,
};
use crate::codebase::dependencies::graph::NodeId;

pub(super) fn unsafe_helper(
    proof: &mut builders::Proof<'_, '_>,
    graph: &DepGraph,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    sql: &SqlCall<'_>,
    binding: &ValueBinding,
) -> bool {
    let ValueExpression::Call(start, end) = binding.value else {
        return false;
    };
    let Some(call) = bindings.value_call(&sql.file, (start, end)) else {
        return false;
    };
    graph.call_sites_in_file(&sql.file).iter().any(|site| {
        if site.offset != start
            || site.caller_id != call.context
            || site.invocation != InvocationKind::Call
            || site.has_unknown_target()
        {
            return false;
        }
        site.targets().any(|target| {
            let Some(NodeId::Symbol {
                file,
                callable_id: Some(owner),
                ..
            }) = target.node
            else {
                return false;
            };
            facts.get_ts_facts(file).is_some_and(|producer| {
                producer.value_flow.returns.iter().any(|(returned, value)| {
                    if returned != owner {
                        return false;
                    }
                    let ValueExpression::Call(inner_start, inner_end) = value else {
                        return false;
                    };
                    bindings
                        .value_call(file, (*inner_start, *inner_end))
                        .is_some_and(|inner| {
                            calls_target(graph, file, inner, file, *owner)
                                || unsafe_append(
                                    proof,
                                    file,
                                    &producer.value_flow,
                                    *owner,
                                    &sql.file,
                                    call,
                                    inner,
                                )
                        })
                })
            })
        })
    })
}

fn unsafe_append(
    proof: &mut builders::Proof<'_, '_>,
    producer: &Path,
    flow: &ValueFlowFacts,
    owner: CallableId,
    caller: &Path,
    outer: &ValueCall,
    inner: &ValueCall,
) -> bool {
    let ValueExpression::Member(base, method) = &inner.callee else {
        return false;
    };
    if method != "append" || !proof.expression(producer, base, true) {
        return false;
    }
    let [argument] = inner.arguments.as_slice() else {
        return true;
    };
    let actual = match argument {
        ValueExpression::Named { name, scope, .. } => flow
            .parameters
            .iter()
            .find(|parameter| {
                parameter.owner == owner
                    && parameter.name == *name
                    && parameter.scope == scope.unwrap_or(0)
                    && parameter.projection.is_empty()
                    && parameter.invalidated_at.is_none()
            })
            .and_then(|parameter| outer.arguments.get(parameter.index)),
        _ => None,
    };
    if let Some(actual) = actual {
        !proof.expression(caller, actual, true)
    } else {
        !proof.expression(producer, argument, true)
    }
}

fn calls_target(
    graph: &DepGraph,
    file: &Path,
    call: &ValueCall,
    producer: &Path,
    owner: CallableId,
) -> bool {
    graph.call_sites_in_file(file).iter().any(|site| {
        site.offset == call.offset
            && site.caller_id == call.context
            && site.invocation == InvocationKind::Call
            && !site.has_unknown_target()
            && site.targets().any(|target| {
                matches!(target.node, Some(NodeId::Symbol {
                    file: target_file,
                    callable_id: Some(target_owner),
                    ..
                }) if target_file.as_ref() == producer && *target_owner == owner)
            })
    })
}
