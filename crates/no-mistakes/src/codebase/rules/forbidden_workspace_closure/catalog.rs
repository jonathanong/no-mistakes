use super::RULE_ID;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use std::path::Path;

pub(super) fn duplicate_setting_findings(root: &Path, sources: &SourceStore) -> Vec<RuleFinding> {
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
