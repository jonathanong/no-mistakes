use super::{Dependency, PackageNode, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use globset::GlobSet;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

pub(super) fn collect_findings_for_package(
    root: &Path,
    start: &str,
    nodes: &BTreeMap<String, PackageNode>,
    workspace_names: &BTreeSet<String>,
    forbidden: &GlobSet,
    source_filter: &crate::codebase::rules::path_filter::RulePathFilter,
    findings: &mut Vec<RuleFinding>,
) {
    let Some(start_node) = nodes.get(start) else {
        return;
    };
    let mut sink = FindingSink {
        root,
        start,
        source_filter,
        findings,
    };
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([(start.to_string(), vec![start.to_string()])]);
    while let Some((package, chain)) = queue.pop_front() {
        if !seen.insert(package.clone()) {
            continue;
        }
        let Some(node) = nodes.get(&package) else {
            continue;
        };
        for dep in &node.deps {
            if let Some(target) = matched_forbidden_name(dep, forbidden) {
                if target != start {
                    let mut dep_chain = chain.clone();
                    dep_chain.push(target.clone());
                    sink.push(&dep_chain, &target, start_node, node);
                }
            }
            let Some(workspace_dep) = dep.workspace_name.as_ref() else {
                continue;
            };
            if workspace_names.contains(workspace_dep) && !seen.contains(workspace_dep) {
                let mut next_chain = chain.clone();
                next_chain.push(workspace_dep.clone());
                queue.push_back((workspace_dep.clone(), next_chain));
            }
        }
    }
}

fn matched_forbidden_name(dep: &Dependency, forbidden: &GlobSet) -> Option<String> {
    if forbidden.is_match(&dep.name) {
        return Some(dep.name.clone());
    }
    dep.resolved_name
        .as_ref()
        .filter(|name| forbidden.is_match(name.as_str()))
        .cloned()
}

struct FindingSink<'a> {
    root: &'a Path,
    start: &'a str,
    source_filter: &'a crate::codebase::rules::path_filter::RulePathFilter,
    findings: &'a mut Vec<RuleFinding>,
}

impl FindingSink<'_> {
    fn push(
        &mut self,
        chain: &[String],
        target: &str,
        start_node: &PackageNode,
        node: &PackageNode,
    ) {
        if !self.source_filter.is_match(&start_node.manifest) {
            return;
        }
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
