use super::RuleFinding;
use crate::codebase::ts_source::{DisableCommentIndex, DisableDirective, SourceStore};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod accounting;
pub(crate) use accounting::read_directive_source;
pub use accounting::{
    suppress_domain_findings_with_source_files, suppress_domain_findings_with_source_locations,
    suppress_domain_findings_with_sources, SuppressedFinding, SuppressionDirective,
    SuppressionDirectiveKind, SuppressionTarget,
};

pub(crate) fn suppress_rule_findings_with_sources_except(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: &SourceStore,
    already_suppressed_rules: &[&str],
) {
    suppress_rule_findings_inner(root, findings, Some(sources), already_suppressed_rules);
}

pub(crate) fn suppress_rule_findings_with_sources(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: &SourceStore,
) {
    suppress_rule_findings_inner(root, findings, Some(sources), &[]);
}

pub(crate) fn suppress_rule_findings_with_source(findings: &mut Vec<RuleFinding>, source: &str) {
    let index = DisableCommentIndex::new(source);
    findings.retain(|finding| {
        finding.rule == super::allow_audit::RULE_ID
            || index
                .matching(
                    &finding.rule,
                    finding.line.try_into().ok(),
                    !markdown_prose(&finding.file),
                    None,
                )
                .is_none()
    });
}

pub(super) fn suppress_rule_findings_inner(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    request_sources: Option<&SourceStore>,
    already_suppressed_rules: &[&str],
) {
    let mut sources = FindingSuppressions::new(root, request_sources);
    findings.retain(|finding| {
        already_suppressed_rules.contains(&finding.rule.as_str())
            || sources
                .matching(&finding.file, &finding.rule, Some(finding.line))
                .is_none()
    });
}

type SharedIndex = Arc<DisableCommentIndex<Arc<str>>>;

struct FindingSuppressions<'a> {
    lexical_root: PathBuf,
    canonical_root: Option<PathBuf>,
    sources: Option<&'a SourceStore>,
    files: HashMap<String, Option<SuppressionFile>>,
}

struct SuppressionFile {
    index: SharedIndex,
    track_quotes: bool,
}

impl<'a> FindingSuppressions<'a> {
    fn new(root: &Path, sources: Option<&'a SourceStore>) -> Self {
        let lexical_root = crate::codebase::ts_source::normalize_discovery_path(root);
        let canonical_root = sources
            .is_none()
            .then(|| std::fs::canonicalize(&lexical_root).ok())
            .flatten();
        Self {
            lexical_root,
            canonical_root,
            sources,
            files: HashMap::new(),
        }
    }

    fn matching(
        &mut self,
        file: &str,
        rule: &str,
        line: Option<usize>,
    ) -> Option<DisableDirective> {
        if rule == super::allow_audit::RULE_ID {
            return None;
        }
        let source = match self.files.get(file) {
            Some(cached) => cached,
            None => {
                let index = self.load(file);
                self.files.entry(file.to_owned()).or_insert(index)
            }
        }
        .as_ref()?;
        source.index.matching(
            rule,
            line.and_then(|line| line.try_into().ok()),
            source.track_quotes,
            self.sources
                .and_then(SourceStore::observer)
                .map(Arc::as_ref),
        )
    }

    fn load(&self, file: &str) -> Option<SuppressionFile> {
        let (candidate, is_absolute) =
            finding_source_candidate(&self.lexical_root, file, self.sources.is_some())?;
        let index = match self.sources {
            Some(sources) => {
                let path = if is_absolute {
                    sources.trusted_regular_path(&candidate)
                } else {
                    sources.validated_regular_path(&self.lexical_root, &candidate)
                }?;
                sources.suppressions_for_path(&path)?
            }
            None => {
                let path = source_path_for_candidate(self.canonical_root.as_deref()?, candidate)?;
                let source = Arc::<str>::from(std::fs::read_to_string(path).ok()?);
                Arc::new(DisableCommentIndex::new(source))
            }
        };
        Some(SuppressionFile {
            index,
            track_quotes: !markdown_prose(file),
        })
    }
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

fn markdown_prose(file: &str) -> bool {
    // `.md` and `.markdown` are prose. `.mdx` keeps JavaScript quote tracking.
    Path::new(file)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
        })
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod invalid_utf8_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod index_tests;
