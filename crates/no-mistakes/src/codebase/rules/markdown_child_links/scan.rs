use super::{matches_group, relative, CompiledGroup, RuleFinding, RULE_ID};
use crate::codebase::rules::markdown_facts::MarkdownFactMap;
use crate::codebase::ts_source::FrozenPathRemapper;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    files: &[PathBuf],
    facts: &MarkdownFactMap,
    groups: &[CompiledGroup],
) -> Result<Vec<RuleFinding>> {
    let known = files.iter().cloned().collect::<BTreeSet<_>>();
    let remapper = FrozenPathRemapper::from_paths(files.iter().cloned());
    let mut findings = Vec::new();
    for group in groups {
        findings.extend(scan_group(root, files, facts, group, &known, &remapper)?);
    }
    Ok(findings)
}

fn scan_group(
    root: &Path,
    files: &[PathBuf],
    facts: &MarkdownFactMap,
    group: &CompiledGroup,
    known: &BTreeSet<PathBuf>,
    remapper: &FrozenPathRemapper,
) -> Result<Vec<RuleFinding>> {
    let parents: Vec<&PathBuf> = files
        .iter()
        .filter(|path| matches_group(&group.parents, &relative(root, path)))
        .collect();
    let children: Vec<&PathBuf> = files
        .iter()
        .filter(|path| matches_group(&group.children, &relative(root, path)))
        .collect();
    let mut linked = BTreeSet::new();
    for parent in &parents {
        let facts = facts.get_for_rule(parent, RULE_ID)?;
        for link in super::links::resolve_parent_links(
            root,
            parent,
            &facts.link_destinations,
            known,
            remapper,
        ) {
            if group.require_whole_file && !link.whole_file {
                continue;
            }
            linked.insert(link.path);
        }
    }
    Ok(children
        .into_iter()
        .filter(|child| !parents.iter().any(|parent| parent == child))
        .filter(|child| !linked.contains(*child))
        .map(|child| missing_link(root, child, group.require_whole_file))
        .collect())
}

fn missing_link(root: &Path, child: &Path, require_whole_file: bool) -> RuleFinding {
    let file = relative(root, child);
    let kind = if require_whole_file {
        "whole-file markdown link"
    } else {
        "markdown link"
    };
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.clone(),
        line: 1,
        message: format!(
            "{file} is not linked from a configured parent markdown file; add a {kind} to this file from a parent"
        ),
        import: None,
        target: Some(file),
    }
}
