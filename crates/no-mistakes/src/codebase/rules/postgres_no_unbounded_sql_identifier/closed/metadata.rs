use super::*;
use crate::codebase::dependencies::extract::ValueBinding;
use crate::codebase::rules::postgres_sql_cross_file::{
    metadata_helper_call, metadata_row_projection,
};

pub(super) fn proves(
    graph: &DepGraph,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    file: &Path,
    flow: &ValueFlowFacts,
    binding: &ValueBinding,
    members: &[String],
) -> bool {
    let bounded = super::bounded(flow, binding, members, false);
    let row = metadata_row_projection(graph, facts, bindings, file, &binding.value, members);
    if members.is_empty() {
        bounded || row || metadata_helper_call(graph, facts, file, &binding.value)
    } else {
        bounded && row
    }
}
