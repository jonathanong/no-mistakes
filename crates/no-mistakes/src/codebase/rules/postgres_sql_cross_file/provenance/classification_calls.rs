use super::classification::IdentifierUse;
use super::*;

pub(super) fn classify_call(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    value_call: &ValueCall,
    out: &mut BTreeSet<Provenance>,
    bindings: &Bindings<'_>,
) {
    with_context(
        graph,
        facts,
        file,
        value_call,
        out,
        bindings,
        IdentifierUse::Value,
    );
}

pub(super) fn with_context(
    graph: &DepGraph,
    facts: &CheckFactMap,
    file: &Path,
    value_call: &ValueCall,
    out: &mut BTreeSet<Provenance>,
    bindings: &Bindings<'_>,
    usage: IdentifierUse<'_>,
) {
    if binding_tag::is_bind_tag(bindings, file, value_call)
        || (usage.members().is_empty()
            && bindings.identifier_return(
                file,
                value_call,
                matches!(usage, IdentifierUse::Identifier),
            ))
        || is_finite_validator(graph, facts, &bindings.routes, file, &value_call.callee)
        || closed_map_lookup(facts, file, value_call)
    {
        return;
    }
    classify_value(graph, facts, file, &value_call.callee, out, bindings);
    for argument in value_call.arguments.iter() {
        classify_value(graph, facts, file, argument, out, bindings);
    }
}

fn closed_map_lookup(facts: &CheckFactMap, file: &Path, value_call: &ValueCall) -> bool {
    let ValueExpression::Member(inner, method) = &value_call.callee else {
        return false;
    };
    if method != "get" {
        return false;
    }
    let ValueExpression::Named { name, .. } = inner.as_ref() else {
        return false;
    };
    facts
        .get_ts_facts(file)
        .is_some_and(|ts| unique_closed_collection(&ts.value_flow, name, RegistryKind::Map))
}
