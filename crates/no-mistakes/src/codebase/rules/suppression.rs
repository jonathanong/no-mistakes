use super::RuleFinding;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) fn suppress_rule_findings(root: &Path, findings: &mut Vec<RuleFinding>) {
    suppress_rule_findings_inner(root, findings, None, &[]);
}

pub(crate) fn suppress_rule_findings_with_sources_except(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: &crate::codebase::ts_source::SourceStore,
    already_suppressed_rules: &[&str],
) {
    suppress_rule_findings_inner(root, findings, Some(sources), already_suppressed_rules);
}

pub(crate) fn suppress_rule_findings_with_source(findings: &mut Vec<RuleFinding>, source: &str) {
    findings.retain(|finding| !finding_is_suppressed(source, finding));
}

fn suppress_rule_findings_inner(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    request_sources: Option<&crate::codebase::ts_source::SourceStore>,
    already_suppressed_rules: &[&str],
) {
    let Some(root) = std::fs::canonicalize(root).ok() else {
        return;
    };
    let mut sources: HashMap<String, Option<std::sync::Arc<str>>> = HashMap::new();
    findings.retain(|finding| {
        if already_suppressed_rules.contains(&finding.rule.as_str()) {
            return true;
        }
        let source = sources.entry(finding.file.clone()).or_insert_with(|| {
            source_path_for_finding(&root, &finding.file).and_then(|path| match request_sources {
                Some(sources) => super::read_source(sources, &path),
                None => std::fs::read_to_string(path)
                    .ok()
                    .map(std::sync::Arc::<str>::from),
            })
        });
        !source
            .as_deref()
            .is_some_and(|source| finding_is_suppressed(source, finding))
    });
}

fn source_path_for_finding(root: &Path, file: &str) -> Option<PathBuf> {
    let path = Path::new(file);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::Prefix(_)
                    | std::path::Component::RootDir
                    | std::path::Component::ParentDir
            )
        })
    {
        return None;
    }
    let candidate = std::fs::canonicalize(root.join(path)).ok()?;
    let metadata = std::fs::metadata(&candidate).ok()?;
    (candidate.starts_with(root) && metadata.is_file()).then_some(candidate)
}

fn finding_is_suppressed(source: &str, finding: &RuleFinding) -> bool {
    let line = finding.line.try_into().ok();
    crate::codebase::ts_source::has_disable_file_comment(source, &finding.rule)
        || line.is_some_and(|line| {
            crate::codebase::ts_source::has_disable_comment(source, line, &finding.rule)
                || crate::codebase::ts_source::has_disable_line_comment(source, line, &finding.rule)
        })
}
