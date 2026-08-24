use super::manifests::{matching_dependencies, Manifest};
use super::workspace;
use super::{Options, RuleFinding, RULE_ID};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) struct CheckContext<'a> {
    pub(super) root: &'a Path,
    pub(super) opts: &'a Options,
    pub(super) manifests: &'a [Manifest],
    pub(super) by_name: &'a BTreeMap<String, Vec<&'a Manifest>>,
    pub(super) fields: &'a [&'a str],
    pub(super) sources: &'a SourceStore,
}

pub(super) fn check_manifest(
    context: &CheckContext,
    manifest: &Manifest,
    findings: &mut Vec<RuleFinding>,
) -> Result<()> {
    let file = relative_slash_path(context.root, &manifest.path);
    let line = workspace::line(&manifest.path, context.sources);
    let target_dirs = match targets(context, manifest) {
        Ok(target_dirs) => target_dirs,
        Err(name) => {
            findings.push(finding(&file, line, format!("{file}: dependency `{name}` matches a configured prefix but has no unique visible package.json target")));
            return Ok(());
        }
    };
    let entries = workspace::entries(&manifest.path, context.sources);
    if wildcard_finding(
        &file,
        line,
        &manifest.dir,
        context.opts,
        context.manifests,
        &entries,
        findings,
    )? {
        return Ok(());
    }
    let expected: BTreeSet<_> = target_dirs
        .values()
        .map(|dir| workspace::relative_from(&manifest.dir, dir))
        .collect();
    let declared = declared_paths(context.opts, context.manifests, &manifest.dir);
    let actual: BTreeSet<_> = entries
        .iter()
        .filter(|entry| declared.contains(entry.as_str()))
        .cloned()
        .collect();
    report_difference(
        &file,
        line,
        &actual,
        &expected,
        "missing nested workspace entries",
        findings,
    );
    report_difference(
        &file,
        line,
        &expected,
        &actual,
        "unused nested workspace entries",
        findings,
    );
    Ok(())
}

fn targets(
    context: &CheckContext,
    manifest: &Manifest,
) -> Result<BTreeMap<String, std::path::PathBuf>, String> {
    let used = context
        .manifests
        .iter()
        .filter(|candidate| candidate.dir.starts_with(&manifest.dir))
        .flat_map(|candidate| {
            matching_dependencies(
                candidate,
                &context.opts.dependency_name_prefixes,
                context.fields,
                context.sources,
            )
        })
        .collect::<BTreeSet<_>>();
    used.iter()
        .map(
            |name| match context.by_name.get(name).filter(|items| items.len() == 1) {
                Some(items) => Ok((name.clone(), items[0].dir.clone())),
                None => Err(name.clone()),
            },
        )
        .collect()
}

fn declared_paths(opts: &Options, manifests: &[Manifest], root: &Path) -> BTreeSet<String> {
    manifests
        .iter()
        .filter(|candidate| {
            candidate.name.as_ref().is_some_and(|name| {
                opts.dependency_name_prefixes
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
            })
        })
        .map(|candidate| workspace::relative_from(root, &candidate.dir))
        .collect()
}

fn wildcard_finding(
    file: &str,
    line: usize,
    root: &Path,
    opts: &Options,
    manifests: &[Manifest],
    entries: &[String],
    findings: &mut Vec<RuleFinding>,
) -> Result<bool> {
    for entry in entries
        .iter()
        .filter(|entry| workspace::contains_wildcard(entry))
    {
        let targets = manifests.iter().filter_map(|candidate| {
            candidate
                .name
                .as_ref()
                .filter(|name| {
                    opts.dependency_name_prefixes
                        .iter()
                        .any(|prefix| name.starts_with(prefix))
                })
                .map(|_| &candidate.dir)
        });
        if workspace::wildcard_targets_dependency(root, entry, targets)? {
            findings.push(finding(file, line, format!("{file}: workspace entry `{entry}` uses a wildcard for a configured dependency package; use its explicit relative path")));
            return Ok(true);
        }
    }
    Ok(false)
}

fn report_difference(
    file: &str,
    line: usize,
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
    label: &str,
    findings: &mut Vec<RuleFinding>,
) {
    let entries: Vec<_> = right.difference(left).cloned().collect();
    if !entries.is_empty() {
        findings.push(finding(
            file,
            line,
            format!("{file}: {label}: {}", entries.join(", ")),
        ));
    }
}

fn finding(file: &str, line: usize, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: None,
    }
}
