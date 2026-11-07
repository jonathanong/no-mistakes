use super::*;
use crate::codebase::rules::postgres_sql_cross_file::CompiledOptions;

#[path = "fragment_builders.rs"]
mod builders;

pub(super) fn proven(
    graph: &DepGraph,
    opts: &CompiledOptions,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    call: &SqlCall<'_>,
    source: &Provenance,
) -> bool {
    let Provenance::Reference {
        name,
        scope,
        offset,
        members,
    } = source
    else {
        return false;
    };
    if members.as_slice() != ["text"] {
        return false;
    }
    let Some(ts) = facts.get_ts_facts(&call.file) else {
        return false;
    };
    builders::Proof::new(graph, opts, bindings, &call.file, &ts.value_flow)
        .binding(*scope, name, *offset, false)
}

#[cfg(test)]
#[path = "fragment_builders_tests.rs"]
mod builder_tests;
