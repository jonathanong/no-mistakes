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
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([(start.to_string(), vec![start.to_string()])]);
    while let Some((package, chain)) = queue.pop_front() {
        if !seen.insert(package.clone()) {
            continue;
        }
        if package != start && forbidden.is_match(&package) {
            if let Some(node) = nodes.get(&package) {
                push_finding(root, start, &chain, &package, node, source_filter, findings);
            }
        }
        let Some(node) = nodes.get(&package) else {
            continue;
        };
        for dep in &node.deps {
            if workspace_names.contains(&dep.name) && !seen.contains(&dep.name) {
                let mut next_chain = chain.clone();
                next_chain.push(dep.name.clone());
                queue.push_back((dep.name.clone(), next_chain));
                continue;
            }
            if let Some(target) = matched_forbidden_name(dep, forbidden) {
                let mut dep_chain = chain.clone();
                dep_chain.push(target.clone());
                push_finding(
                    root,
                    start,
                    &dep_chain,
                    &target,
                    node,
                    source_filter,
                    findings,
                );
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

fn push_finding(
    root: &Path,
    start: &str,
    chain: &[String],
    target: &str,
    node: &PackageNode,
    source_filter: &crate::codebase::rules::path_filter::RulePathFilter,
    findings: &mut Vec<RuleFinding>,
) {
    if !source_filter.is_match(&node.manifest) {
        return;
    }
    let file = relative_slash_path(root, &node.manifest);
    let chain_text = chain.join(" -> ");
    findings.push(RuleFinding {
        rule: RULE_ID.to_string(),
        file,
        line: 1,
        message: format!(
            "{start} reaches forbidden package '{target}' through package.json dependency chain: {chain_text}. Remove the dependency or move it outside this workspace closure."
        ),
        import: Some(chain_text),
        target: Some(target.to_string()),
    });
}
