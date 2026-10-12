use super::{Dependency, PackageNode, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use globset::GlobSet;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

pub(super) struct Traversal<'a> {
    root: &'a Path,
    nodes: &'a BTreeMap<String, PackageNode>,
    workspace_names: &'a BTreeSet<String>,
    forbidden: &'a GlobSet,
    source_filter: &'a crate::codebase::rules::path_filter::RulePathFilter,
    observer: Option<&'a crate::diagnostics::InvocationObserver>,
}

impl<'a> Traversal<'a> {
    pub(super) fn new(
        root: &'a Path,
        nodes: &'a BTreeMap<String, PackageNode>,
        workspace_names: &'a BTreeSet<String>,
        forbidden: &'a GlobSet,
        source_filter: &'a crate::codebase::rules::path_filter::RulePathFilter,
        observer: Option<&'a crate::diagnostics::InvocationObserver>,
    ) -> Self {
        Self {
            root,
            nodes,
            workspace_names,
            forbidden,
            source_filter,
            observer,
        }
    }

    pub(super) fn collect_findings_for_package<'b>(
        &'b self,
        start: &'b str,
        findings: &mut Vec<RuleFinding>,
    ) {
        let Some(start_node) = self.nodes.get(start) else {
            return;
        };
        if !self.source_filter.is_match(&start_node.manifest) {
            return;
        }
        let mut sink = FindingSink {
            root: self.root,
            start,
            findings,
        };
        // Claim nodes when enqueued so the first breadth-first route remains the
        // winning route while merged/duplicate routes never carry their own path.
        let mut predecessors: BTreeMap<&'b str, Option<&'b str>> = BTreeMap::from([(start, None)]);
        let mut queue = VecDeque::from([start]);
        while let Some(package) = queue.pop_front() {
            let Some(node) = self.nodes.get(package) else {
                continue;
            };
            for dep in &node.deps {
                if let Some(target) = matched_forbidden_name(dep, self.forbidden) {
                    if target != start {
                        let mut chain = reconstruct_chain(start, package, &predecessors);
                        if let Some(observer) = self.observer {
                            observer
                                .increment("forbidden_workspace_closure.path_reconstructions", 1);
                        }
                        chain.push(target);
                        sink.push(&chain, target, node);
                    }
                }
                let Some(workspace_dep) = dep.workspace_name.as_deref() else {
                    continue;
                };
                if self.workspace_names.contains(workspace_dep)
                    && !predecessors.contains_key(workspace_dep)
                {
                    predecessors.insert(workspace_dep, Some(package));
                    queue.push_back(workspace_dep);
                }
            }
        }
    }
}

fn reconstruct_chain<'a>(
    start: &'a str,
    package: &'a str,
    predecessors: &BTreeMap<&'a str, Option<&'a str>>,
) -> Vec<&'a str> {
    let mut chain = vec![package];
    let mut current = package;
    while current != start {
        let previous = predecessors
            .get(current)
            .copied()
            .flatten()
            .expect("every queued workspace package has a predecessor");
        chain.push(previous);
        current = previous;
    }
    chain.reverse();
    chain
}

fn matched_forbidden_name<'a>(dep: &'a Dependency, forbidden: &GlobSet) -> Option<&'a str> {
    if forbidden.is_match(&dep.name) {
        return Some(dep.name.as_str());
    }
    dep.resolved_name
        .as_ref()
        .filter(|name| forbidden.is_match(name.as_str()))
        .map(String::as_str)
}

struct FindingSink<'a> {
    root: &'a Path,
    start: &'a str,
    findings: &'a mut Vec<RuleFinding>,
}

impl FindingSink<'_> {
    fn push(&mut self, chain: &[&str], target: &str, node: &PackageNode) {
        let file = relative_slash_path(self.root, &node.manifest);
        let chain_text = chain.join(" -> ");
        self.findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file,
            line: 1,
            message: format!(
                "{} reaches forbidden package '{}' through package.json dependency chain: {chain_text}. Remove the dependency or move it outside this workspace closure.",
                self.start, target
            ),
            import: Some(chain_text),
            target: Some(target.to_string()),
        });
    }
}
