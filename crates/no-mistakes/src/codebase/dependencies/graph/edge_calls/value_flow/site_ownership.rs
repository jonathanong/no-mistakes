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

fn function_keys(site: &ResolvedCallSite) -> impl Iterator<Item = FunctionKey> + '_ {
    node_function_keys(site.invocation, site.target_nodes())
}
fn node_function_keys<'a>(
    invocation: InvocationKind,
    nodes: impl Iterator<Item = &'a NodeId> + 'a,
) -> impl Iterator<Item = FunctionKey> + 'a {
    nodes.filter_map(move |node| {
        if invocation == InvocationKind::Membership {
            return None;
        }
        match node {
            NodeId::Symbol {
                file,
                callable_id: Some(id),
                ..
            } => Some((file.to_path_buf(), *id)),
            _ => None,
        }
    })
}
fn site_argument(
    site: &ResolvedCallSite,
    indexes: &CallableResolutionIndexes,
    facts: &dyn TsFactLookup,
) -> ParameterInput {
    ParameterInput::from_site(site, indexes, facts)
}

fn site_source(inputs: &GraphEdgeBuildInputs<'_>, site: &ResolvedCallSite) -> NodeId {
    site.caller.as_deref().map_or_else(
        || NodeId::file_in(&inputs.interner, &site.file),
        |caller| NodeId::scoped_in(&inputs.interner, &site.file, caller, site.caller_id),
    )
}

// The serial incoming update needs canonical old nodes, not the old target's names.
// Compare before dropping that target: changed targets can share the same node.
enum PreviousCallNodes {
    Unchanged,
    Single(Option<NodeId>),
    Alternatives(std::sync::Arc<ResolvedCallAlternatives>),
}
impl PreviousCallNodes {
    fn project(previous: PreviousResolution, site: &ResolvedCallSite) -> Self {
        if previous.0 == site.target && previous.1 == site.target_node {
            return Self::Unchanged;
        }
        match previous.0 {
            ResolvedCallTarget::Alternatives(alternatives) => Self::Alternatives(alternatives),
            _ => Self::Single(previous.1),
        }
    }
    fn changed(&self) -> bool {
        !matches!(self, Self::Unchanged)
    }
    fn target_nodes(&self) -> ResolvedCallNodes<'_> {
        match self {
            Self::Alternatives(alternatives) => ResolvedCallNodes {
                single: None,
                alternatives: alternatives.nodes.iter(),
            },
            Self::Single(node) => ResolvedCallNodes {
                single: node.as_ref(),
                alternatives: [].iter(),
            },
            Self::Unchanged => ResolvedCallNodes {
                single: None,
                alternatives: [].iter(),
            },
        }
    }
}
