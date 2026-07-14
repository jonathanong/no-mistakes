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

pub(crate) fn suppress_rule_findings_with_sources(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: &crate::codebase::ts_source::SourceStore,
) {
    suppress_rule_findings_inner(root, findings, Some(sources), &[]);
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
    let lexical_root = crate::codebase::ts_source::normalize_discovery_path(root);
    let canonical_root = request_sources
        .is_none()
        .then(|| std::fs::canonicalize(&lexical_root).ok())
        .flatten();
    if request_sources.is_none() && canonical_root.is_none() {
        return;
    }
    let mut sources: HashMap<String, Option<std::sync::Arc<str>>> = HashMap::new();
    findings.retain(|finding| {
        if already_suppressed_rules.contains(&finding.rule.as_str()) {
            return true;
        }
        let source = sources.entry(finding.file.clone()).or_insert_with(|| {
            let relative = safe_relative_finding_path(&finding.file)?;
            let candidate = lexical_root.join(relative);
            let path = match request_sources {
                Some(sources) => sources.validated_regular_path(&lexical_root, &candidate),
                None => source_path_for_candidate(
                    canonical_root
                        .as_deref()
                        .expect("raw suppression canonicalizes root"),
                    candidate,
                ),
            }?;
            match request_sources {
                Some(sources) => super::read_source(sources, &path),
                None => std::fs::read_to_string(path)
                    .ok()
                    .map(std::sync::Arc::<str>::from),
            }
        });
        !source
            .as_deref()
            .is_some_and(|source| finding_is_suppressed(source, finding))
    });
}

fn safe_relative_finding_path(file: &str) -> Option<&Path> {
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
    Some(path)
}

fn source_path_for_candidate(canonical_root: &Path, candidate: PathBuf) -> Option<PathBuf> {
    let canonical_candidate = std::fs::canonicalize(&candidate).ok()?;
    let metadata = std::fs::metadata(&canonical_candidate).ok()?;
    (canonical_candidate.starts_with(canonical_root) && metadata.is_file()).then_some(candidate)
}

fn finding_is_suppressed(source: &str, finding: &RuleFinding) -> bool {
    let line = finding.line.try_into().ok();
    crate::codebase::ts_source::has_disable_file_comment(source, &finding.rule)
        || line.is_some_and(|line| {
            crate::codebase::ts_source::has_disable_comment(source, line, &finding.rule)
                || crate::codebase::ts_source::has_disable_line_comment(source, line, &finding.rule)
        })
}
