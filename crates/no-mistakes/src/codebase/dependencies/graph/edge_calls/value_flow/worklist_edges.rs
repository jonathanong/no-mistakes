fn replace_call_edges(
    edges: &mut Vec<Edge>,
    base: &[ResolvedCallSite],
    sites: &[ResolvedCallSite],
    processed: &[usize],
    source: impl Fn(&ResolvedCallSite) -> NodeId,
) {
    // Initial edges contain one entry per physical occurrence (before the
    // canonical adjacency dedupe). Remove exactly that occurrence's old Call
    // edges; identical calls and implicit constructor routes must survive.
    let mut removed: FxHashMap<Edge, usize> = fx_map();
    for index in processed {
        let site = &base[*index];
        for node in site.target_nodes() {
            *removed
                .entry((source(site), node.clone(), EdgeKind::Call))
                .or_default() += 1;
        }
    }
    if !removed.is_empty() {
        edges.retain(|edge| {
            if let Some(count) = removed.get_mut(edge) {
                if *count > 0 {
                    *count -= 1;
                    return false;
                }
            }
            true
        });
    }
    for index in processed {
        let site = &sites[*index];
        for node in site.target_nodes() {
            edges.push((source(site), node.clone(), EdgeKind::Call));
        }
    }
}
