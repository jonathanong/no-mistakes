use super::{extract, SetSpec, RULE_ID, TS_CALL_FIRST_STRING_ARGUMENT};
use crate::codebase::rules::{analysis_policy::AnalysisDiagnostics, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::EffectiveRuleAnalysis;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn supports(kind: &str) -> bool {
    matches!(
        kind,
        "yaml-sequence" | "yaml-string-selector" | "path-regex-capture"
    )
}

pub(super) fn record_unsupported(
    root: &Path,
    spec: &SetSpec,
    target_roots: &[PathBuf],
    sources: &SourceStore,
    policy: EffectiveRuleAnalysis,
    output: &mut AnalysisDiagnostics,
) -> Result<()> {
    let paths = extract::resolve_spec_files(root, &spec.file, target_roots);
    // Empty call targets are configuration failures at max, before source access.
    if spec.kind == TS_CALL_FIRST_STRING_ARGUMENT && spec.target.is_empty() {
        for path in &paths {
            output.findings.push(RuleFinding {
                rule: RULE_ID.into(),
                file: relative_slash_path(root, path),
                line: 1,
                message: format!(
                    "finite set '{}' requires a non-empty target for kind '{}'",
                    spec.name, TS_CALL_FIRST_STRING_ARGUMENT
                ),
                import: None,
                target: None,
            });
        }
        return Ok(());
    }
    // Effort narrows extraction, not the requirement that declared inputs exist.
    for path in &paths {
        sources
            .read_path(path)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    }
    let file = relative_slash_path(root, &paths[0]);
    output.unanalyzable(policy.unanalyzable, RuleFinding {
        rule: RULE_ID.into(),
        file: file.clone(),
        line: 1,
        message: format!("{file}: finite set '{}' with kind '{}' is unanalyzable at effort low; use yaml-sequence, yaml-string-selector, or path-regex-capture, or use effort max for this application", spec.name, spec.kind),
        import: None,
        target: Some("unanalyzable".into()),
    });
    Ok(())
}
