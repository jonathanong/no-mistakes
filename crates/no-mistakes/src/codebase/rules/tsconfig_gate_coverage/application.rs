use super::{command_scan, is_tsconfig_path, RuleFinding, RULE_ID};
use std::collections::{BTreeMap, BTreeSet};

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct Options {
    /// Reasoned exemptions keyed by the repository-relative tsconfig path.
    pub(super) allow_projects: BTreeMap<String, String>,
}

pub(super) fn scan_application(
    opts: &Options,
    tracked: &BTreeSet<String>,
    candidates: &BTreeSet<String>,
    ci_projects: &BTreeSet<String>,
    local_projects: &BTreeSet<String>,
    config_file: &str,
) -> Vec<RuleFinding> {
    let ci_projects = resolve_gate_projects_against_tracked(ci_projects, tracked);
    let local_projects = resolve_gate_projects_against_tracked(local_projects, tracked);
    let (allowlist, mut findings) = validate_allowlist(opts, tracked, config_file);
    for project in candidates {
        if allowlist.contains(project) {
            continue;
        }
        if !ci_projects.contains(project) {
            findings.push(project_finding(
                project,
                format!(
                    "{project}: tsconfig has no CI typecheck registration; add a static `tsc --noEmit --project {project}` command to a configured workflow, or add a reasoned `allowProjects` entry"
                ),
            ));
        }
        if !local_projects.contains(project) {
            findings.push(project_finding(
                project,
                format!(
                    "{project}: tsconfig has no local typecheck registration; add an `always: true` `checks.commands` entry with `fileArgs: none` that statically runs `tsc --noEmit --project {project}`, or add a reasoned `allowProjects` entry"
                ),
            ));
        }
    }
    findings
}

/// Resolve the directory form accepted by `tsc --project` only when the
/// request's tracked tsconfig inventory proves that interpretation. This keeps
/// static command parsing path-only while correctly handling directories such
/// as `app.json` that a filename-suffix heuristic would misclassify.
pub(super) fn resolve_gate_projects_against_tracked(
    gate_projects: &BTreeSet<String>,
    tracked: &BTreeSet<String>,
) -> BTreeSet<String> {
    gate_projects
        .iter()
        .map(|project| {
            if tracked.contains(project) {
                return project.clone();
            }
            let directory_config = if project == "." {
                "tsconfig.json".to_string()
            } else {
                format!("{project}/tsconfig.json")
            };
            if tracked.contains(&directory_config) {
                directory_config
            } else {
                project.clone()
            }
        })
        .collect()
}

fn validate_allowlist(
    opts: &Options,
    tracked: &BTreeSet<String>,
    config_file: &str,
) -> (BTreeSet<String>, Vec<RuleFinding>) {
    let mut normalized = BTreeMap::<String, String>::new();
    let mut accepted = BTreeSet::new();
    let mut findings = Vec::new();
    for (raw_path, reason) in &opts.allow_projects {
        let Some(path) = command_scan::normalize_repo_relative(raw_path) else {
            findings.push(config_finding(
                config_file,
                raw_path,
                format!(
                    "allowProjects entry `{raw_path}` must be a static repository-relative tsconfig path"
                ),
            ));
            continue;
        };
        if !is_tsconfig_path(&path) {
            findings.push(config_finding(
                config_file,
                raw_path,
                format!("allowProjects entry `{raw_path}` is not a tsconfig path"),
            ));
            continue;
        }
        if reason.trim().is_empty() {
            findings.push(config_finding(
                config_file,
                raw_path,
                format!("allowProjects entry `{raw_path}` must include a non-empty reason"),
            ));
            continue;
        }
        if let Some(first) = normalized.insert(path.clone(), raw_path.clone()) {
            findings.push(config_finding(
                config_file,
                raw_path,
                format!(
                    "allowProjects entries `{first}` and `{raw_path}` normalize to the same path `{path}`"
                ),
            ));
            continue;
        }
        if !tracked.contains(&path) {
            findings.push(config_finding(
                config_file,
                raw_path,
                format!(
                    "stale allowProjects entry `{raw_path}` does not name a tracked tsconfig; remove it"
                ),
            ));
            continue;
        }
        accepted.insert(path);
    }
    (accepted, findings)
}

pub(super) fn project_finding(file: &str, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target: None,
    }
}

fn config_finding(file: &str, target: &str, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
