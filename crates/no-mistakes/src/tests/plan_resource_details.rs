/// The impact BFS runs in the reverse direction, while resource provenance is
/// keyed by its canonical `consumer → resource` graph edge.
pub(crate) fn resource_edge_detail(
    graph: &DepGraph,
    reverse_from: &NodeId,
    reverse_to: &NodeId,
    kind: EdgeKind,
    root: &Path,
) -> Option<ImpactEdgeDetail> {
    if kind != EdgeKind::Resource {
        return None;
    }
    let (NodeId::File(consumer), NodeId::File(resource)) = (reverse_from, reverse_to) else {
        return None;
    };
    let call_sites = graph
        .resource_edge_details(consumer, resource)?
        .iter()
        .map(|site| ResourceCallSite {
            call_kind: site.call_kind.clone(),
            line: u32::try_from(site.line).unwrap_or(u32::MAX),
        })
        .collect();
    Some(ImpactEdgeDetail::Resource {
        consumer_file: relative_path(root, consumer),
        call_sites,
    })
}
