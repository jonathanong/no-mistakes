use super::*;
use std::collections::VecDeque;
#[derive(Default)]
pub(super) struct Outcome {
    pub assertion: bool,
    pub uncertain: bool,
    unmodeled: bool,
}
pub(super) fn analyze(
    root: &NodeId,
    options: &Options,
    graph: &DepGraph,
    facts: &dyn TsFactLookup,
    sites: &FxHashMap<NodeId, Vec<&ResolvedCallSite>>,
    contracts: &[ownership::Contract],
) -> Outcome {
    let mut out = Outcome::default();
    let mut seen = FxHashSet::default();
    seen.insert(root.clone());
    let mut queue = VecDeque::from([(root.clone(), 0usize)]);
    while let Some((node, depth)) = queue.pop_front() {
        let file = node.as_file().expect("callable file");
        if unavailable(file, facts) || generator(&node, facts) {
            out.uncertain = true;
            continue;
        }
        let mut targets = FxHashSet::default();
        let mut live_fs = false;
        let node_sites = sites.get(&node).map_or(&[][..], Vec::as_slice);
        for site in node_sites
            .iter()
            .copied()
            .filter(|site| eligible(site, facts))
        {
            // Calling a generator creates an iterator; neither a configured
            // selector nor a contract makes that occurrence execute its body.
            if site.target_nodes().any(|target| generator(target, facts)) {
                out.uncertain = true;
                continue;
            }
            if options
                .assertions
                .iter()
                .any(|assertion| assertion.matches(site))
            {
                out.assertion = true;
                continue;
            }
            let modeled = contracts
                .iter()
                .filter(|contract| contract.from == node && contract.via.matches(site))
                .collect::<Vec<_>>();
            out.uncertain |= modeled.iter().any(|contract| {
                unavailable(contract.to.as_file().expect("callable file"), facts)
                    || generator(&contract.to, facts)
            });
            targets.extend(modeled.iter().map(|contract| contract.to.clone()));
            targets.extend(site.target_nodes().cloned());
            // External calls are opaque boundaries unless explicitly modeled. Node `fs`
            // is modeled I/O; known globals remain syntactically classified, while
            // dynamic targets abstain.
            if site.has_unknown_target() {
                out.uncertain = true;
            } else if modeled.is_empty() && external::unmodeled_external(site) {
                out.unmodeled = true;
            }
            if modeled.is_empty() && external::fs_effect(site) {
                live_fs = true;
                out.uncertain |= external::callback_argument(site, facts);
            }
        }
        // Reachability of a synthetic callback is not recorded; a live `fs` call
        // plus any callback in the same callable stays uncertain.
        out.uncertain |= live_fs && external::has_callback_site(node_sites);
        for (target, kind) in graph.dependencies_of_node(&node).into_iter().flatten() {
            if !targets.contains(target)
                || !matches!(kind, EdgeKind::Call | EdgeKind::AssertionOwner)
                || seen.contains(target)
            {
                continue;
            }
            if depth >= options.max_depth.unwrap_or(64)
                || seen.len() >= options.max_nodes.unwrap_or(10000)
            {
                out.uncertain = true;
                continue;
            }
            seen.insert(target.clone());
            queue.push_back((target.clone(), depth + 1));
        }
    }
    // Unmodeled modules are uncertain only when no assertion owner was reached.
    if out.unmodeled && !out.assertion {
        out.uncertain = true;
    }
    out
}
