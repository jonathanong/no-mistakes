use super::*;
use std::collections::VecDeque;
#[derive(Default)]
pub(super) struct Outcome {
    pub assertion: bool,
    pub(super) causes: std::collections::BTreeSet<uncertainty::Cause>,
    pub(super) source_failures: std::collections::BTreeMap<std::path::PathBuf, String>,
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
        let source_unavailable = unavailable(file, facts);
        if source_unavailable {
            out.record_source_failure(file, facts);
        }
        if source_unavailable {
            continue;
        }
        if generator(&node, facts) {
            out.causes
                .insert(uncertainty::Cause::Generator(node.clone()));
            continue;
        }
        let mut targets = FxHashSet::default();
        let mut live_fs = Vec::new();
        let node_sites = sites.get(&node).map_or(&[][..], Vec::as_slice);
        for site in node_sites
            .iter()
            .copied()
            .filter(|site| eligible(site, facts))
        {
            for failed in graph.failed_call_sources(site) {
                out.record_source_failure(failed, facts);
            }
            // Calling a generator creates an iterator; neither a configured
            // selector nor a contract makes that occurrence execute its body.
            if site.target_nodes().any(|target| generator(target, facts)) {
                out.dispatch(site, "generator body execution is not modeled");
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
            for contract in &modeled {
                let file = contract.to.as_file().expect("callable file");
                if unavailable(file, facts) {
                    out.record_source_failure(file, facts);
                }
                if generator(&contract.to, facts) {
                    out.causes
                        .insert(uncertainty::Cause::Generator(contract.to.clone()));
                }
            }
            targets.extend(modeled.iter().map(|contract| contract.to.clone()));
            targets.extend(site.target_nodes().cloned());
            // External calls are opaque boundaries unless explicitly modeled. Node `fs`
            // is modeled I/O; known globals remain syntactically classified, while
            // dynamic targets abstain.
            if let Some(callbacks) = builtins::classify(site, node_sites, facts) {
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
                if external::callback_argument(site, facts) {
                    out.dispatch(site, "callback argument execution is not modeled");
                }
            }
        }
        // Reachability of a synthetic callback is not recorded; a live `fs` call
        // plus any callback in the same callable stays uncertain.
        if external::has_callback_site(node_sites) {
            for site in live_fs {
                out.dispatch(site, "callback execution alongside fs I/O is not modeled");
            }
        }
        for (target, kind) in graph.dependencies_of_node(&node).into_iter().flatten() {
            if !targets.contains(target)
                || !matches!(kind, EdgeKind::Call | EdgeKind::AssertionOwner)
                || seen.contains(target)
            {
                continue;
            }
            let max_depth = options.max_depth.unwrap_or(64);
            let max_nodes = options.max_nodes.unwrap_or(10000);
            if depth >= max_depth || seen.len() >= max_nodes {
                if depth >= max_depth {
                    out.limit("maxDepth", max_depth, &node);
                }
                if seen.len() >= max_nodes {
                    out.limit("maxNodes", max_nodes, &node);
                }
                continue;
            }
            seen.insert(target.clone());
            queue.push_back((target.clone(), depth + 1));
        }
    }
    out
}
