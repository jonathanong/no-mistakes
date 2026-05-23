use super::types::{GlobMatcher, Options};
use super::{RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_resolver::{normalize_path, ImportResolver};
use crate::codebase::ts_source::relative_slash_path;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub(super) fn namespace_import_findings(
    root: &Path,
    project_root: &Path,
    shared: &CheckFactMap,
    story_files: &BTreeSet<PathBuf>,
    resolver: &ImportResolver<'_>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for file in story_files {
        let Some(facts) = shared
            .ts
            .get(file)
            .and_then(|facts| facts.storybook.as_ref())
        else {
            continue;
        };
        for import in &facts.used_runtime_imports {
            if !import.namespace {
                continue;
            }
            let Some(resolved) = resolver
                .resolve(&import.source, file)
                .map(|p| normalize_path(&p))
            else {
                continue;
            };
            if !resolved.starts_with(project_root) {
                continue;
            }
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: relative_slash_path(root, file),
                line: import.line as usize,
                message: format!(
                    "Storybook namespace import `{}` is ambiguous for component coverage; import covered components by name or default export.",
                    import.local
                ),
                import: Some(import.source.clone()),
                target: None,
            });
        }
    }
    findings
}

pub(super) fn stale_or_blank_allow_findings(
    root: &Path,
    project_root: &Path,
    opts: &Options,
    component_keys: &HashSet<String>,
    _allow_files: &GlobMatcher,
    shared: &CheckFactMap,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for (key, reason) in &opts.allow_components {
        if reason.trim().is_empty() || !component_keys.contains(key) {
            findings.push(component_allow_finding(root, project_root, key, reason));
        }
    }
    for (pattern, reason) in &opts.allow_files {
        if reason.trim().is_empty() {
            findings.push(file_allow_finding(pattern, "must include a reason"));
            continue;
        }
        let matcher = GlobMatcher::new(std::iter::once(pattern));
        let matched = shared.files().iter().any(|path| {
            path.starts_with(project_root)
                && matcher.is_match(&relative_slash_path(project_root, path))
        });
        if !matched {
            findings.push(file_allow_finding(
                pattern,
                "does not match any project file",
            ));
        }
    }
    findings
}

fn component_allow_finding(
    root: &Path,
    project_root: &Path,
    key: &str,
    reason: &str,
) -> RuleFinding {
    let file = key.split_once('#').map(|(file, _)| file).unwrap_or(key);
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, &project_root.join(file)),
        line: 1,
        message: if reason.trim().is_empty() {
            format!("Storybook component opt-out `{key}` must include a reason.")
        } else {
            format!("Storybook component opt-out `{key}` does not match a selected component.")
        },
        import: None,
        target: Some(key.to_string()),
    }
}

fn file_allow_finding(pattern: &str, reason: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: pattern.to_string(),
        line: 1,
        message: format!("Storybook file opt-out `{pattern}` {reason}."),
        import: None,
        target: None,
    }
}
