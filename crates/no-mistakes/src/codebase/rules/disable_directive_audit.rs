//! Aggregate-only suppression hygiene. Findings are appended after filtering.
use super::suppression::{SuppressionDirective, SuppressionDirectiveKind};
use super::{RuleFinding, SuppressedFinding};
mod consumed;
mod postgres;
mod scopes;
use crate::codebase::ts_source::{
    audit_directives_for_path, relative_slash_path, DisableDirective, SourceStore,
};
use crate::config::v2::{DisableDirectivePolicy, NoMistakesConfig};
use anyhow::Result;
#[doc(hidden)]
pub use consumed::ConsumedDisableDirective;
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Reuses the request inventory, source store, application scopes, and consumed
/// directive provenance. Never rediscovers files or parses a TS/JS program.
#[doc(hidden)]
pub fn audit(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    suppressed: &[SuppressedFinding],
) -> Result<Vec<RuleFinding>> {
    Ok(PreparedDisableDirectives::prepare(root, config, files, sources)?.findings(suppressed))
}

/// Comment metadata owned by the request, prepared before source inputs freeze.
#[doc(hidden)]
pub struct PreparedDisableDirectives {
    root: PathBuf,
    comments: Vec<PreparedComment>,
}

struct PreparedComment {
    file: String,
    rule: String,
    directive: SuppressionDirective,
    status: DirectiveStatus,
}

enum DirectiveStatus {
    Enabled,
    Disabled,
    Unknown,
    IneffectivePlacement,
}

impl PreparedDisableDirectives {
    pub fn prepare(
        root: &Path,
        config: &NoMistakesConfig,
        files: &[PathBuf],
        sources: &SourceStore,
    ) -> Result<Self> {
        let audit_hygiene = config.audits_disable_directives() && !config.rules.is_empty();
        let scopes = scopes::comment_scopes(root, config, files, audit_hygiene)?;

        let comments = scopes
            .par_iter()
            .flat_map_iter(|(path, enabled)| {
                let Some(source) = super::suppression::read_directive_source(sources, path) else {
                    return Vec::new();
                };
                let file = relative_slash_path(root, path);
                audit_directives_for_path(&source, path)
                    .into_iter()
                    .filter_map(|(named, ineffective)| {
                        let status = if audit_hygiene && ineffective {
                            DirectiveStatus::IneffectivePlacement
                        } else if !known_rule(&named.rule) {
                            DirectiveStatus::Unknown
                        } else if !audit_hygiene {
                            return None;
                        } else if enabled.contains(named.rule.as_str()) {
                            DirectiveStatus::Enabled
                        } else {
                            DirectiveStatus::Disabled
                        };
                        Some(PreparedComment {
                            file: file.clone(),
                            rule: named.rule,
                            directive: project(named.directive),
                            status,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok(Self {
            root: root.to_path_buf(),
            comments,
        })
    }

    #[doc(hidden)]
    pub fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }

    /// Finalization only matches retained comments to consumed provenance.
    pub fn findings(&self, suppressed: &[SuppressedFinding]) -> Vec<RuleFinding> {
        let items = suppressed
            .iter()
            .map(|i| (i.source_file.as_str(), i.rule.as_str(), &i.directive));
        self.findings_for_locations(items, &BTreeSet::new())
    }

    /// Complete internal consumption, independent of deduplicated public findings.
    pub fn findings_from_consumed(
        &self,
        consumed: &[ConsumedDisableDirective],
    ) -> Vec<RuleFinding> {
        self.findings_excluding_unparsed(consumed, &BTreeSet::new())
    }

    pub(super) fn findings_excluding_unparsed(
        &self,
        consumed: &[ConsumedDisableDirective],
        unparsed_files: &BTreeSet<String>,
    ) -> Vec<RuleFinding> {
        let items = consumed
            .iter()
            .map(|i| (i.source_file.as_str(), i.rule.as_str(), &i.directive));
        self.findings_for_locations(items, unparsed_files)
    }

    fn findings_for_locations<'a>(
        &self,
        consumed: impl Iterator<Item = (&'a str, &'a str, &'a SuppressionDirective)>,
        unparsed_files: &BTreeSet<String>,
    ) -> Vec<RuleFinding> {
        let used: BTreeSet<_> = consumed
            .map(|(file, rule, directive)| {
                (
                    relative_slash_path(&self.root, Path::new(file)),
                    rule,
                    directive.clone(),
                )
            })
            .collect();
        let mut findings = self.comments.iter().filter_map(|comment| {
            if matches!(comment.status, DirectiveStatus::Enabled) && unparsed_files.contains(&comment.file) { return None }
            if matches!(comment.status, DirectiveStatus::Enabled | DirectiveStatus::Disabled) && used.contains(&(comment.file.clone(), comment.rule.as_str(), comment.directive.clone())) { return None }
            let (rule, detail) = match comment.status {
                DirectiveStatus::Enabled => ("unused-disable-directive", format!("directive for `{}` suppressed no findings in this invocation", comment.rule)),
                DirectiveStatus::Disabled => ("invalid-disable-directive", format!("directive names disabled or out-of-scope rule `{}`", comment.rule)),
                DirectiveStatus::IneffectivePlacement => ("invalid-disable-directive", format!("directive for `{}` in a Rust documentation comment cannot suppress findings; use a plain `//` comment instead", comment.rule)),
                DirectiveStatus::Unknown => ("invalid-disable-directive", format!("directive names unknown rule `{}`", comment.rule)),
            };
            Some(RuleFinding { causes: Vec::new(),
                rule: rule.to_owned(), file: comment.file.clone(), line: comment.directive.line,
                message: format!("{detail}; remove or correct the comment so suppressions cannot hide stale rule intent"),
                import: None, target: Some(comment.rule.clone()),
            })
        }).collect();
        super::sort_findings(&mut findings);
        findings
    }
}

fn project(directive: DisableDirective) -> SuppressionDirective {
    let (kind, line) = match directive {
        DisableDirective::File { line } => (SuppressionDirectiveKind::File, line),
        DisableDirective::Line { line } => (SuppressionDirectiveKind::Line, line),
        DisableDirective::NextLine { line } => (SuppressionDirectiveKind::NextLine, line),
    };
    SuppressionDirective {
        kind,
        line: line as usize,
    }
}

mod known_rules;
use known_rules::known_rule;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod catalog_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod block_doc_tests;
