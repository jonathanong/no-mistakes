// Only admitted occurrences need immutable initial resolution across visits.
struct InitialResolutions {
    slots: Vec<usize>,
    values: Vec<InitialResolution>,
}
struct InitialResolution {
    target: ResolvedCallTarget,
    target_node: Option<NodeId>,
    alternatives: Vec<ResolvedCallSite>,
    typed_from: Vec<(String, String)>,
    returned_from: Vec<(String, String)>,
    returned_member_from: Vec<(String, String, String)>,
}
impl InitialResolutions {
    fn new(sites: &[ResolvedCallSite], pending: &FxHashSet<usize>) -> Self {
        let mut slots = vec![usize::MAX; if pending.is_empty() { 0 } else { sites.len() }];
        let mut values = Vec::with_capacity(pending.len());
        for &index in pending {
            slots[index] = values.len();
            let site = &sites[index];
            values.push(InitialResolution {
                target: site.target.clone(),
                target_node: site.target_node.clone(),
                alternatives: site.alternatives.clone(),
                typed_from: site.typed_from.clone(),
                returned_from: site.returned_from.clone(),
                returned_member_from: site.returned_member_from.clone(),
            });
        }
        #[cfg(all(test, feature = "test-instrumentation"))]
        snapshot_tests::record(
            values.len(),
            slots.capacity() * size_of::<usize>()
                + values.capacity() * size_of::<InitialResolution>(),
            0,
        );
        Self { slots, values }
    }
    fn get(&self, index: usize) -> &InitialResolution {
        &self.values[self.slots[index]]
    }
}
impl InitialResolution {
    fn target_nodes(&self) -> ResolvedCallNodes<'_> {
        match &self.target {
            ResolvedCallTarget::Alternatives(alternatives) => ResolvedCallNodes {
                single: None,
                alternatives: alternatives.nodes.iter(),
            },
            _ => ResolvedCallNodes {
                single: self.target_node.as_ref(),
                alternatives: [].iter(),
            },
        }
    }
}

// Physical metadata stays in place. Only resolution-owned fields are reset.
type PreviousResolution = (ResolvedCallTarget, Option<NodeId>);
fn reset_resolution(site: &mut ResolvedCallSite, base: &InitialResolution) -> PreviousResolution {
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

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "resolution_snapshot_tests.rs"]
mod snapshot_tests;
