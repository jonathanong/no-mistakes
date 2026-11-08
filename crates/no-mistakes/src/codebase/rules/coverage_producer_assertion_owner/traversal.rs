use super::*;
use std::collections::VecDeque;
#[derive(Default)]
pub(super) struct Outcome {
    pub assertion: bool,
    pub(super) causes: std::collections::BTreeSet<uncertainty::Cause>,
    pub(super) source_failures: std::collections::BTreeMap<std::path::PathBuf, String>,
}

struct Context<'a> {
    options: &'a Options,
    graph: &'a DepGraph,
    facts: &'a dyn TsFactLookup,
    contracts: &'a [ownership::Contract],
}

pub(super) fn analyze(
    root: &NodeId,
    options: &Options,
    graph: &DepGraph,
    facts: &dyn TsFactLookup,
    sites: &FxHashMap<NodeId, Vec<&ResolvedCallSite>>,
    contracts: &[ownership::Contract],
) -> Outcome {
    let cx = Context {
        options,
        graph,
        facts,
        contracts,
    };
    let mut out = Outcome::default();
    let mut seen = FxHashSet::default();
    seen.insert(root.clone());
    let mut queue = VecDeque::from([(root.clone(), 0usize)]);
    while let Some((node, depth)) = queue.pop_front() {
        let file = node.as_file().expect("callable file");
        if unavailable(file, facts) {
            out.record_source_failure(file, facts);
            continue;
        }
        if generator(&node, facts) {
            out.causes
                .insert(uncertainty::Cause::Generator(node.clone()));
            continue;
        }
        let node_sites = sites.get(&node).map_or(&[][..], Vec::as_slice);
        let targets = visit_node(&cx, &node, node_sites, &mut out);
        let state = (&mut seen, &mut queue);
        enqueue(&cx, (&node, depth), &targets, state, &mut out);
    }
    out
}

fn visit_node(
    cx: &Context<'_>,
    node: &NodeId,
    node_sites: &[&ResolvedCallSite],
    out: &mut Outcome,
) -> FxHashSet<NodeId> {
    let mut targets = FxHashSet::default();
    let mut live_fs = Vec::new();
    for site in node_sites
        .iter()
        .copied()
        .filter(|site| eligible(site, cx.facts))
    {
        let scope = (node, node_sites);
        visit_site(cx, scope, site, (&mut targets, &mut live_fs), out);
    }
    // Reachability of a synthetic callback is not recorded; a live `fs` call
    // plus any callback in the same callable stays uncertain.
    if external::has_callback_site(node_sites) {
        for site in live_fs {
            out.dispatch(site, "callback execution alongside fs I/O is not modeled");
        }
    }
    targets
}

fn visit_site<'a>(
    cx: &Context<'_>,
    (node, node_sites): (&NodeId, &[&'a ResolvedCallSite]),
    site: &'a ResolvedCallSite,
    (targets, live_fs): (&mut FxHashSet<NodeId>, &mut Vec<&'a ResolvedCallSite>),
    out: &mut Outcome,
) {
    for failed in cx.graph.failed_call_sources(site) {
        out.record_source_failure(failed, cx.facts);
    }
    // Calling a generator creates an iterator; neither a configured
    // selector nor a contract makes that occurrence execute its body.
    if site
        .target_nodes()
        .any(|target| generator(target, cx.facts))
    {
        out.dispatch(site, "generator body execution is not modeled");
        return;
    }
    if cx
        .options
        .assertions
        .iter()
        .any(|assertion| assertion.matches(site))
    {
        out.assertion = true;
        return;
    }
    let modeled = model_contracts(cx, node, site, out);
    targets.extend(modeled.iter().map(|contract| contract.to.clone()));
    targets.extend(site.target_nodes().cloned());
    // External calls are opaque boundaries unless explicitly modeled. Node `fs`
    // is modeled I/O; known globals remain syntactically classified, while
    // dynamic targets abstain.
    if let Some(callbacks) = builtins::classify(site, node_sites, cx.facts) {
        for callback in callbacks.executed {
            targets.extend(callback.target_nodes().cloned());
        }
        for callback in callbacks.unresolved {
            out.dispatch(
                site,
                format!("built-in callback `{callback}` is not statically resolved"),
            );
        }
    } else if site.has_unknown_target() {
        out.dispatch(site, "callee target is unknown");
    } else if modeled.is_empty() && external::unmodeled_external(site) {
        out.dispatch(site, "external call has no ownership model");
    }
    if modeled.is_empty() && external::fs_effect(site) {
        live_fs.push(site);
        if external::callback_argument(site, cx.facts) {
            out.dispatch(site, "callback argument execution is not modeled");
        }
    }
}

fn model_contracts<'c>(
    cx: &Context<'c>,
    node: &NodeId,
    site: &ResolvedCallSite,
    out: &mut Outcome,
) -> Vec<&'c ownership::Contract> {
    let modeled = cx
        .contracts
        .iter()
        .filter(|contract| contract.from == *node && contract.via.matches(site))
        .collect::<Vec<_>>();
    for contract in &modeled {
        let file = contract.to.as_file().expect("callable file");
        if unavailable(file, cx.facts) {
            out.record_source_failure(file, cx.facts);
        }
        if generator(&contract.to, cx.facts) {
            out.causes
                .insert(uncertainty::Cause::Generator(contract.to.clone()));
        }
    }
    modeled
}

type Frontier<'a> = (&'a mut FxHashSet<NodeId>, &'a mut VecDeque<(NodeId, usize)>);

fn enqueue(
    cx: &Context<'_>,
    (node, depth): (&NodeId, usize),
    targets: &FxHashSet<NodeId>,
    (seen, queue): Frontier<'_>,
    out: &mut Outcome,
) {
    let max_depth = cx.options.max_depth.unwrap_or(64);
    let max_nodes = cx.options.max_nodes.unwrap_or(10000);
    for (target, kind) in cx.graph.dependencies_of_node(node).into_iter().flatten() {
        if !targets.contains(target)
            || !matches!(kind, EdgeKind::Call | EdgeKind::AssertionOwner)
            || seen.contains(target)
        {
            continue;
        }
        if depth >= max_depth || seen.len() >= max_nodes {
            if depth >= max_depth {
                out.limit("maxDepth", max_depth, node);
            }
            if seen.len() >= max_nodes {
                out.limit("maxNodes", max_nodes, node);
            }
            continue;
        }
        seen.insert(target.clone());
        queue.push_back((target.clone(), depth + 1));
    }
}
