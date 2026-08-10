use super::RuleFinding;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod accounting;
pub use accounting::{
    suppress_domain_findings_with_sources, SuppressedFinding, SuppressionDirective,
    SuppressionDirectiveKind, SuppressionTarget,
};

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
            let (candidate, is_absolute) =
                finding_source_candidate(&lexical_root, &finding.file, request_sources.is_some())?;
            let path = match request_sources {
                Some(sources) if is_absolute => sources.trusted_regular_path(&candidate),
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

fn finding_source_candidate(
    root: &Path,
    file: &str,
    allow_absolute: bool,
) -> Option<(PathBuf, bool)> {
    let path = Path::new(file);
    if crate::codebase::ts_source::is_portably_absolute_path(path) {
        return allow_absolute.then(|| (path.to_path_buf(), true));
    }
    if path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Prefix(_) | std::path::Component::RootDir
        )
    }) || (!allow_absolute
        && path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir)))
    {
        return None;
    }
    Some((root.join(path), false))
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

fn matching_directive(
    source: &str,
    rule: &str,
    line: Option<usize>,
) -> Option<SuppressionDirective> {
    use crate::codebase::ts_source::DisableDirective;

    match crate::codebase::ts_source::matching_disable_directive(
        source,
        line.and_then(|line| u32::try_from(line).ok()),
        rule,
    )? {
        DisableDirective::File { line } => Some(SuppressionDirective {
            kind: SuppressionDirectiveKind::File,
            line: line as usize,
        }),
        DisableDirective::Line { line } => Some(SuppressionDirective {
            kind: SuppressionDirectiveKind::Line,
            line: line as usize,
        }),
        DisableDirective::NextLine { line } => Some(SuppressionDirective {
            kind: SuppressionDirectiveKind::NextLine,
            line: line as usize,
        }),
    }
}
