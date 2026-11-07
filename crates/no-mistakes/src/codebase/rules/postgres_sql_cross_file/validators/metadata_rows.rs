use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{InvocationKind, ValueExpression};
use crate::codebase::dependencies::graph::{DepGraph, NodeId, TsFactLookup};
use crate::codebase::rules::postgres_sql_cross_file::Bindings;
use std::path::Path;

pub(crate) fn projection(
    graph: &DepGraph,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    file: &Path,
    value: &ValueExpression,
    members: &[String],
) -> bool {
    let (value, field) = match (value, members) {
        (ValueExpression::Member(base, field), []) => (base.as_ref(), field.as_str()),
        (ValueExpression::Call(_, _), [field]) => (value, field.as_str()),
        _ => return false,
    };
    let value = match value {
        ValueExpression::Named {
            name,
            scope: Some(scope),
            offset,
            ..
        } => {
            let Some(binding) = bindings
                .value(file, *scope, name, *offset)
                .filter(|binding| binding.immutable)
            else {
                return false;
            };
            &binding.value
        }
        value => value,
    };
    let ValueExpression::Call(offset, _) = value else {
        return false;
    };
    let sites = graph.call_sites_in_file(file);
    let start = sites.partition_point(|site| site.offset < *offset);
    let mut sites = sites[start..]
        .iter()
        .take_while(|site| site.offset == *offset)
        .filter(|site| site.invocation == InvocationKind::Call);
    let Some(site) = sites.next() else {
        return false;
    };
    if sites.next().is_some() || graph.failed_call_sources(site).next().is_some() {
        return false;
    }
    let Some(NodeId::Symbol {
        file: owner_file,
        callable_id: Some(owner),
        ..
    }) = &site.target_node
    else {
        return false;
    };
    if owner_file.as_ref() != file {
        return false;
    }
    let Some(ts) = facts.get_ts_facts(file) else {
        return false;
    };
    if ts.parse_error.is_some()
        || ts.operational_error.is_some()
        || ts.value_flow.incomplete_reason.is_some()
    {
        return false;
    }
    ts.value_flow.metadata_row_helpers.iter().any(|helper| {
        helper.owner == *owner
            && helper.fields.iter().any(|key| key == field)
            && !ts.value_flow.registry_mutations.iter().any(|mutation| {
                mutation.name == helper.source.0 && mutation.scope == helper.source.1
            })
    })
}

#[cfg(test)]
#[path = "metadata_rows_tests.rs"]
mod tests;
