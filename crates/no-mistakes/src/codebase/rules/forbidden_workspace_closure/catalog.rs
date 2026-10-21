use super::RULE_ID;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use std::path::Path;

pub(super) fn duplicate_setting_findings(
    root: &Path,
    sources: &SourceStore,
    source_filter: &crate::codebase::rules::path_filter::RulePathFilter,
) -> Vec<RuleFinding> {
    // Catalog data stays readable for traversal. The diagnostic is owned by
    // the workspace file, so a narrowed include must not report it.
    if !source_filter.is_match(&root.join("pnpm-workspace.yaml")) {
        return Vec::new();
    }
    let Ok(Some(resolved)) =
        crate::codebase::workspaces::resolve_pnpm_include_at_root(root, sources)
    else {
        return Vec::new();
    };
    resolved
        .duplicates
        .into_iter()
        .filter(|setting| setting == "catalog" || setting.starts_with("catalogs"))
        .map(|setting| RuleFinding {
            rule: RULE_ID.to_string(),
            file: "pnpm-workspace.yaml".to_string(),
            line: 1,
            message: format!(
                "pnpm-workspace.yaml: setting \"{setting}\" is claimed by more than one pnpm source"
            ),
            import: None,
            target: Some(setting),
        })
        .collect()
}
