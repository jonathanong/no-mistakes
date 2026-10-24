use super::*;
use std::collections::VecDeque;
#[derive(Default)]
pub(super) struct Outcome {
    pub assertion: bool,
    pub uncertain: bool,
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
        for site in sites
            .get(&node)
            .into_iter()
            .flatten()
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
            // External calls are opaque boundaries unless explicitly modeled. Known
            // globals remain syntactically classified, while dynamic targets abstain.
            if site.has_unknown_target()
                || (modeled.is_empty()
                    && site.targets().any(|candidate| {
                        matches!(
                            candidate.target,
                            KnownCallTargetRef::ModuleExport {
                                repository_target: None,
                                ..
                            }
                        )
                    }))
            {
                out.uncertain = true;
            }
        }
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
    out
}
