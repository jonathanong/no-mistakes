use super::*;
use crate::codebase::analysis_session::AnalysisSession;

pub(super) fn require_sources(
    graph: &DepGraph,
    facts: &CheckFactMap,
    filter: &super::super::path_filter::RulePathFilter,
    session: Option<&AnalysisSession>,
) -> Result<()> {
    let roots: Vec<_> = facts
        .graph_file_universe()
        .iter()
        .filter(|file| filter.is_match(file))
        .map(NodeId::file)
        .collect();
    require_files(graph, roots, session)
}

pub(super) fn require_reachable<'a>(
    graph: &DepGraph,
    sites: impl IntoIterator<Item = &'a ResolvedCallSite>,
    excluded: &[NodeId],
    session: Option<&AnalysisSession>,
) -> Result<()> {
    if graph.parse_errors().next().is_none() {
        return Ok(());
    }
    let sites: Vec<_> = sites.into_iter().collect();
    let roots: Vec<_> = sites
        .iter()
        .flat_map(|site| site.target_nodes().cloned())
        .collect();
    let reachable = reachable_sources(graph, roots, excluded);
    let files: std::collections::BTreeSet<_> =
        reachable.iter().filter_map(|node| node.as_file()).collect();
    let mut failed: std::collections::BTreeSet<_> = sites
        .iter()
        .flat_map(|site| graph.failed_call_sources(site))
        .cloned()
        .collect();
    failed.extend(
        files
            .into_iter()
            .flat_map(|file| graph.call_sites_in_file(file))
            .filter(|site| reachable.contains(&source_node(site)))
            .flat_map(|site| graph.failed_call_sources(site))
            .cloned(),
    );
    require_files(
        graph,
        reachable
            .into_iter()
            .chain(failed.into_iter().map(NodeId::file)),
        session,
    )
}

fn require_files(
    graph: &DepGraph,
    nodes: impl IntoIterator<Item = NodeId>,
    session: Option<&AnalysisSession>,
) -> Result<()> {
    let files: std::collections::BTreeSet<_> = nodes
        .into_iter()
        .filter_map(|node| node.as_file().map(ToOwned::to_owned))
        .collect();
    for file in files {
        let result: Result<()> = graph.parse_error(&file).map_or(Ok(()), |error| {
            Err(
                crate::codebase::check_outcome::CheckInputFailure::source(file.clone(), error)
                    .into(),
            )
        });
        if let Some(session) = session {
            session.capture_check_result(RULE_ID, result)?;
        } else {
            result?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

fn reachable_sources(
    graph: &DepGraph,
    roots: Vec<NodeId>,
    excluded: &[NodeId],
) -> crate::fx::FxHashSet<NodeId> {
    let mut reached = crate::fx::FxHashSet::default();
    let mut pending: std::collections::VecDeque<_> = roots.into_iter().collect();
    while let Some(node) = pending.pop_front() {
        if excluded.binary_search(&node).is_ok() || !reached.insert(node.clone()) {
            continue;
        }
        if let Some(edges) = graph.dependencies_of_node(&node) {
            pending.extend(
                edges
                    .iter()
                    .filter(|(_, kind)| {
                        *kind == crate::codebase::dependencies::graph::EdgeKind::Call
                    })
                    .map(|(node, _)| node.clone()),
            );
        }
    }
    reached
}
