//! Holes that sit outside an identifier position.
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
                .filter(|item| retained(bindings, &call.file, item)),
        );
    }
}

/// A plain function parameter is the known bind value. Every other provenance
/// stays, and later export resolution drops a finite member.
fn retained(bindings: &Bindings<'_>, file: &Path, item: &Provenance) -> bool {
    match item {
        Provenance::Reference { name, scope, .. } => !bindings.plain_parameter(file, *scope, name),
        _ => true,
    }
}

#[cfg(test)]
mod harness;
#[cfg(test)]
mod tests;
