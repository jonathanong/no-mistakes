use super::keys::yaml_top_level_key_line;
use super::{finding, RuleFinding, FIX};
use crate::codebase::ts_source::SourceStore;
use std::path::Path;

pub(super) fn included_override_findings(
    root: &Path,
    path: &Path,
    source: &str,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let root_yaml = crate::codebase::ts_resolver::normalize_path(&root.join("pnpm-workspace.yaml"));
    if crate::codebase::ts_resolver::normalize_path(path) != root_yaml {
        return Vec::new();
    }
    let Ok(resolved) =
        crate::codebase::workspaces::resolve_pnpm_include(root, source, |included| {
            sources.read_path(included).ok()
        })
    else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    if resolved
        .duplicates
        .iter()
        .any(|setting| setting == "overrides")
    {
        findings.push(finding(
            root,
            path,
            1,
            "setting \"overrides\" is claimed by more than one pnpm source",
        ));
    }
    for issue in resolved
        .load_errors
        .iter()
        .filter(|issue| issue.affects("overrides"))
    {
        findings.push(finding(root, path, 1, issue.message.clone()));
    }
    for included in resolved.included_override_paths {
        let Ok(text) = sources.read_path(&included) else {
            continue;
        };
        findings.push(finding(
            root,
            &included,
            yaml_top_level_key_line(&text, "overrides"),
            format!("top-level \"overrides\" is banned. {FIX}"),
        ));
    }
    findings
}
