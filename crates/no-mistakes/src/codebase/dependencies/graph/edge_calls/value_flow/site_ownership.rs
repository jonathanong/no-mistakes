// Physical metadata stays in place. Only resolution-owned fields are reset.
type PreviousResolution = (ResolvedCallTarget, Option<NodeId>);
fn reset_resolution(site: &mut ResolvedCallSite, base: &ResolvedCallSite) -> PreviousResolution {
    let target = std::mem::replace(&mut site.target, base.target.clone());
    let node = std::mem::replace(&mut site.target_node, base.target_node.clone());
    site.alternatives.clone_from(&base.alternatives);
    site.typed_from.clone_from(&base.typed_from);
    site.returned_from.clone_from(&base.returned_from);
    site.returned_member_from
        .clone_from(&base.returned_member_from);
    (target, node)
}
fn swap_resolution(site: &mut ResolvedCallSite, previous: &mut PreviousResolution) {
    std::mem::swap(&mut site.target, &mut previous.0);
    std::mem::swap(&mut site.target_node, &mut previous.1);
}
fn selected_sites<'a>(
    sites: &'a mut [ResolvedCallSite],
    batch: &[usize],
) -> Vec<(usize, &'a mut ResolvedCallSite)> {
    let mut tail = sites;
    let mut next = 0;
    batch
        .iter()
        .map(|&index| {
            // Batch is sorted and unique because it was drained from the pending set.
            let (_, selected) = std::mem::take(&mut tail).split_at_mut(index - next);
            let (site, rest) = selected.split_first_mut().unwrap();
            tail = rest;
            next = index + 1;
            (index, site)
        })
        .collect()
}
