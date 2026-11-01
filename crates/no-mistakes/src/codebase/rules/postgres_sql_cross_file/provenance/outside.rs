//! Env and config holes that sit outside an identifier position.
use super::super::SqlCall;
use super::bindings::Bindings;
use super::classify;
use super::Provenance;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::dependencies::graph::DepGraph;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn retain(
    graph: &DepGraph,
    facts: &CheckFactMap,
    call: &SqlCall<'_>,
    bindings: &Bindings<'_>,
    holes: &[&ValueExpression],
    out: &mut BTreeSet<Provenance>,
) {
    for hole in holes {
        let mut extra = BTreeSet::new();
        classify(graph, facts, &call.file, hole, &mut extra, bindings);
        out.extend(
            extra
                .into_iter()
                .filter(|item| derived(bindings, &call.file, item)),
        );
    }
}

fn derived(bindings: &Bindings<'_>, file: &Path, item: &Provenance) -> bool {
    match item {
        Provenance::ProcessEnv(_) => true,
        // Export resolution drops a finite namespace member.
        Provenance::Namespace { .. } => true,
        Provenance::Reference {
            name,
            scope,
            offset,
            members,
        } => {
            bindings.env_alias(file, *scope, name, *offset)
                || bindings.query_projection(file, *scope, name, *offset)
                || bindings.imported_value(file, name)
                || (bindings.flows_member_unbounded(file, *scope, name, *offset, members)
                    && !bindings.parameter(file, name))
                || (bindings.flows_unbounded(file, *scope, name, *offset)
                    && !bindings.parameter(file, name))
        }
    }
}

#[cfg(test)]
mod harness;
#[cfg(test)]
mod tests;
