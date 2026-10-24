use super::inputs::{active_names, cooldown, lockfile_keys};
use super::lockfile::package_name_from_lock_key;
use super::policy::{check_with_complete_names, ExcludeEntry, FileKind, Issue, Snapshot};
use super::{rel, Options, RuleFinding, RULE_ID};
use crate::codebase::ts_source::SourceStore;
use serde_yaml::Value as Yaml;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let by_rel: HashMap<String, &PathBuf> =
        files.iter().map(|path| (rel(root, path), path)).collect();
    let Some(workspace) = by_rel.get(opts.workspace_yaml()) else {
        return Vec::new();
    };
    let source = match sources.read_path(workspace) {
        Ok(source) => source,
        Err(error) => {
            return vec![super::super::manifest_analysis::unavailable(
                root, workspace, RULE_ID, error,
            )]
        }
    };
    let yaml = match serde_yaml::from_str::<Yaml>(&source) {
        Ok(Yaml::Mapping(map)) => Yaml::Mapping(map),
        Ok(_) => return Vec::new(),
        Err(error) => {
            return vec![super::super::manifest_analysis::unavailable(
                root,
                workspace,
                RULE_ID,
                format!("failed to parse YAML: {error}"),
            )];
        }
    };
    let mut findings = Vec::new();
    let lockfile_keys = match by_rel.get(opts.lockfile_path()) {
        Some(path) => match lockfile_keys(sources, path) {
            Ok(keys) => keys,
            Err(message) => {
                findings.push(super::super::manifest_analysis::unavailable(
                    root, path, RULE_ID, message,
                ));
                None
            }
        },
        None => None,
    };
    let mut active_names = active_names(root, files, sources, &mut findings);
    let complete_names = findings.is_empty();
    if let Some(keys) = &lockfile_keys {
        for key in keys {
            if let Some(name) = package_name_from_lock_key(key) {
                active_names.insert(name);
            }
        }
    }
    let (exclude, included_findings) =
        super::included_exclude::included_exclude(root, workspace, sources, exclude_entries(&yaml));
    findings.extend(included_findings);
    let cooldown = match by_rel.get(opts.dependabot_path()) {
        Some(path) => match cooldown(sources, path) {
            Ok(value) => value,
            Err(error) => {
                findings.push(super::super::manifest_analysis::unavailable(
                    root, path, RULE_ID, error,
                ));
                None
            }
        },
        None => None,
    };
    let snapshot = Snapshot {
        exclude,
        cooldown,
        active_names,
        lockfile_keys,
    };
    findings.extend(
        check_with_complete_names(opts, &snapshot, complete_names)
            .into_iter()
            .map(|issue| issue_finding(root, opts, &by_rel, issue)),
    );
    findings
}

fn exclude_entries(yaml: &Yaml) -> Vec<ExcludeEntry> {
    match yaml.get("minimumReleaseAgeExclude") {
        Some(Yaml::Sequence(seq)) => seq
            .iter()
            .map(|entry| match entry.as_str() {
                Some(name) => ExcludeEntry::Name(name.to_string()),
                None => ExcludeEntry::Other,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn issue_finding(
    root: &Path,
    opts: &Options,
    by_rel: &HashMap<String, &PathBuf>,
    issue: Issue,
) -> RuleFinding {
    let rel_path = match issue.file {
        FileKind::Workspace => opts.workspace_yaml().to_string(),
        FileKind::Dependabot => opts.dependabot_path().to_string(),
        FileKind::Lockfile => opts.lockfile_path().to_string(),
    };
    let file = by_rel
        .get(&rel_path)
        .map(|path| rel(root, path))
        .unwrap_or(rel_path);
    finding(
        file.clone(),
        format!("{file}: {}", issue.message),
        &issue.target,
    )
}

fn finding(file: String, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file,
        line: 1,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
