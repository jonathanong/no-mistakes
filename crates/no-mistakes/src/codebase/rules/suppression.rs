use super::RuleFinding;
use serde::Serialize;
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

/// A domain finding projected into the common suppression contract.
///
/// The check runner creates these only after all domain analyzers have consumed
/// the request-scoped `SourceStore`; this keeps suppression from introducing a
/// second source-read path.
pub struct SuppressionTarget<'a> {
    pub domain: &'static str,
    pub rule: &'a str,
    pub file: &'a str,
    pub line: Option<usize>,
    pub reason: &'a str,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuppressedFinding {
    pub domain: String,
    pub rule: String,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// The deterministic diagnostic that would have been emitted.
    pub reason: String,
    pub directive: SuppressionDirective,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuppressionDirective {
    pub kind: SuppressionDirectiveKind,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SuppressionDirectiveKind {
    File,
    Line,
    NextLine,
}

/// Retain unsuppressed findings and return deterministic accounting for the
/// findings removed by a valid directive. A missing source is deliberately not
/// treated as a suppression: it must not hide a diagnostic.
pub fn suppress_domain_findings_with_sources<T>(
    root: &Path,
    findings: &mut Vec<T>,
    sources: &crate::codebase::ts_source::SourceStore,
    describe: impl Fn(&T) -> SuppressionTarget<'_>,
) -> Vec<SuppressedFinding> {
    let lexical_root = crate::codebase::ts_source::normalize_discovery_path(root);
    let mut cached_sources: HashMap<String, Option<std::sync::Arc<str>>> = HashMap::new();
    let mut suppressed = Vec::new();
    findings.retain(|finding| {
        let target = describe(finding);
        let source = cached_sources
            .entry(target.file.to_string())
            .or_insert_with(|| {
                let (candidate, is_absolute) =
                    finding_source_candidate(&lexical_root, target.file, true)?;
                let path = if is_absolute {
                    sources.trusted_regular_path(&candidate)
                } else {
                    sources.validated_regular_path(&lexical_root, &candidate)
                }?;
                super::read_source(sources, &path)
            });
        let Some(directive) = source
            .as_deref()
            .and_then(|source| matching_directive(source, target.rule, target.line))
        else {
            return true;
        };
        suppressed.push(SuppressedFinding {
            domain: target.domain.to_string(),
            rule: target.rule.to_string(),
            file: target.file.to_string(),
            line: target.line,
            reason: target.reason.to_string(),
            directive,
        });
        false
    });
    suppressed.sort();
    suppressed.dedup();
    suppressed
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
    if crate::codebase::ts_source::has_disable_file_comment(source, rule) {
        return Some(SuppressionDirective {
            kind: SuppressionDirectiveKind::File,
            line: file_directive_line(source, rule),
        });
    }
    let line = u32::try_from(line?).ok()?;
    if crate::codebase::ts_source::has_disable_line_comment(source, line, rule) {
        return Some(SuppressionDirective {
            kind: SuppressionDirectiveKind::Line,
            line: line as usize,
        });
    }
    crate::codebase::ts_source::has_disable_comment(source, line, rule).then_some(
        SuppressionDirective {
            kind: SuppressionDirectiveKind::NextLine,
            line: line.saturating_sub(1) as usize,
        },
    )
}

fn file_directive_line(source: &str, rule: &str) -> usize {
    source
        .trim_start_matches('\u{FEFF}')
        .lines()
        .enumerate()
        .find_map(|(index, line)| {
            let trimmed = line.trim();
            let directive = trimmed
                .strip_prefix("//")
                .or_else(|| trimmed.strip_prefix('#'))
                .or_else(|| trimmed.strip_prefix("--"))?
                .trim();
            let rest = directive.strip_prefix("no-mistakes-disable-file ")?;
            directive_rule_part_matches(rest.trim(), rule).then_some(index + 1)
        })
        // `has_disable_file_comment` already established that a valid directive
        // exists. This fallback is only for unusual leading block-comment forms.
        .unwrap_or(1)
}

fn directive_rule_part_matches(rule_part: &str, rule: &str) -> bool {
    rule_part.strip_prefix(rule).is_some_and(|suffix| {
        suffix.is_empty() || suffix.starts_with(':') || suffix.starts_with(char::is_whitespace)
    })
}
