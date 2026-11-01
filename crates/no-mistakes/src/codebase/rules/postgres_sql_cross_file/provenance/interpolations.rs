//! Interpolations of an executor call that are candidate identifier sources.
use super::bindings::Bindings;
use super::{binding_tag, classify, classify_call, contains, Provenance};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::rules::postgres_sql_cross_file::SqlCall;
use std::collections::BTreeSet;

pub(crate) fn interpolations(
    graph: &DepGraph,
    facts: &CheckFactMap,
    call: &SqlCall<'_>,
    bindings: &Bindings<'_>,
) -> BTreeSet<Provenance> {
    let mut out = BTreeSet::new();
    let Some(ts) = facts.get_ts_facts(&call.file) else {
        return out;
    };
    let calls = &ts.value_flow.calls;
    let binds = binding_tag::bind_spans(bindings, &call.file, calls);
    for value_call in calls {
        let span = (value_call.offset, value_call.end);
        if !contains(call.span, span) || binds.iter().any(|bind| contains(*bind, span)) {
            continue;
        }
        if span == call.span {
            let tagged = value_call.arguments.first().is_some_and(|first| {
                binding_tag::leads_with_bind_tag(bindings, &call.file, calls, first)
            });
            for argument in value_call
                .arguments
                .iter()
                .take(if tagged { 1 } else { usize::MAX })
            {
                classify(graph, facts, &call.file, argument, &mut out, bindings);
            }
            continue;
        }
        classify_call(graph, facts, &call.file, value_call, &mut out, bindings);
    }
    out
}
