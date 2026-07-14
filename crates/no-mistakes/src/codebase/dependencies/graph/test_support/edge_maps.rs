use super::*;
use std::collections::HashSet;
use std::path::PathBuf;

pub(super) fn edge_index_from_test_maps(
    forward: EdgeMap,
    reverse: EdgeMap,
) -> EdgeIndex<NodeId, EdgeKind> {
    // Some graph unit tests intentionally provide only the direction under
    // test. Preserve the historical constructor's union semantics here while
    // keeping the production constructor on the direct, consistency-checked path.
    let forward_nodes = forward.keys().cloned().collect::<Vec<_>>();
    let mut edges = forward
        .into_iter()
        .flat_map(|(from, adjacent)| {
            adjacent
                .into_iter()
                .map(move |(to, kind)| (from.clone(), to, kind))
        })
        .collect::<HashSet<_>>();
    edges.extend(reverse.into_iter().flat_map(|(to, adjacent)| {
        adjacent
            .into_iter()
            .map(move |(from, kind)| (from, to.clone(), kind))
    }));

    let mut canonical_forward = forward_nodes
        .into_iter()
        .map(|node| (node, Vec::new()))
        .collect::<EdgeMap>();
    let mut canonical_reverse = EdgeMap::new();
    for (from, to, kind) in edges {
        canonical_forward
            .entry(from.clone())
            .or_default()
            .push((to.clone(), kind));
        canonical_reverse.entry(to).or_default().push((from, kind));
    }
    edge_index_from_maps(canonical_forward, canonical_reverse)
}

pub(crate) fn add_distinct_worker_file_edges(
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
    worker_file: &PathBuf,
    processor_file: &PathBuf,
    queue_job: &NodeId,
) {
    if worker_file != processor_file {
        forward
            .entry(queue_job.clone())
            .or_default()
            .push((NodeId::File(worker_file.clone()), EdgeKind::QueueWorker));
        reverse
            .entry(NodeId::File(worker_file.clone()))
            .or_default()
            .push((queue_job.clone(), EdgeKind::QueueWorker));
    }
}

pub(in super::super) fn add_queue_edges(
    root: &Path,
    resolver: &ImportResolver<'_>,
    files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
    config_options: Option<&GraphConfigOptions>,
    forward: &mut EdgeMap,
    reverse: &mut EdgeMap,
) {
    super::super::merge_edges(
        forward,
        reverse,
        super::super::collect_queue_edges(root, resolver, files, facts, config_options),
    );
}
