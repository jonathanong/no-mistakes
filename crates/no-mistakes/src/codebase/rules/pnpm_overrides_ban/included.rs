use super::keys::yaml_top_level_key_line;
use super::scan::finding;
use super::{ClassifiedFinding, FIX};
use crate::codebase::ts_source::SourceStore;
use std::path::Path;

pub(super) fn included_override_findings(
    root: &Path,
    path: &Path,
    source: &str,
    sources: &SourceStore,
    files: &[std::path::PathBuf],
) -> Vec<ClassifiedFinding> {
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
    let (mut findings, failed_sources) =
        super::unavailable_includes::collect(root, source, sources, files);
    let root_selected = files.iter().any(|candidate| {
        crate::codebase::ts_resolver::normalize_path(candidate)
            == crate::codebase::ts_resolver::normalize_path(path)
    });
    if root_selected
        && resolved
            .duplicates
            .iter()
            .any(|setting| setting == "overrides")
    {
        findings.push(ClassifiedFinding::Definite(finding(
            root,
            path,
            1,
            "setting \"overrides\" is claimed by more than one pnpm source",
        )));
    }
    if root_selected {
        for issue in resolved
            .load_errors
            .iter()
            .filter(|issue| issue.affects("overrides"))
        {
            // A selected source has a more precise read/parse diagnostic; retain
            // independent invalid grants and escape errors on the selected root.
            if failed_sources
                .iter()
                .any(|source| issue.message.contains(&format!("\"{source}\"")))
            {
                continue;
            }
            let mut note = finding(root, path, 1, issue.message.clone());
            note.target = Some("unanalyzable".to_owned());
            findings.push(ClassifiedFinding::Unanalyzable(note));
        }
    }
    for included in resolved.included_override_paths {
        let included_path = crate::codebase::ts_resolver::normalize_path(&included);
        let allowed = files.iter().any(|candidate| {
            crate::codebase::ts_resolver::normalize_path(candidate) == included_path
        });
        if !allowed {
            continue;
        }
        let text = match sources.read_path(&included) {
            Ok(text) => text,
            Err(error) => {
                findings.push(super::super::manifest_analysis::unavailable(
                    root,
                    &included,
                    super::RULE_ID,
                    error,
                ));
                continue;
            }
        };
        findings.push(ClassifiedFinding::Definite(finding(
            root,
            &included,
            yaml_top_level_key_line(&text, "overrides"),
            format!("top-level \"overrides\" is banned. {FIX}"),
        )));
    }
    findings
}
