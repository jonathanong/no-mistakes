//! Aggregate-only suppression hygiene. Findings are appended after filtering.
use super::suppression::{SuppressionDirective, SuppressionDirectiveKind};
use super::{RuleFinding, SuppressedFinding};
mod consumed;
use crate::codebase::ts_source::{
    disable_directives, relative_slash_path, DisableDirective, SourceStore,
};
use crate::config::v2::{DisableDirectivePolicy, NoMistakesConfig};
use anyhow::Result;
#[doc(hidden)]
pub use consumed::ConsumedDisableDirective;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
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
}

impl PreparedDisableDirectives {
    pub fn prepare(
        root: &Path,
        config: &NoMistakesConfig,
        files: &[PathBuf],
        sources: &SourceStore,
    ) -> Result<Self> {
        if config.report_unused_disable_directives == DisableDirectivePolicy::Ignore {
            return Ok(Self {
                root: root.to_path_buf(),
                comments: Vec::new(),
            });
        }
        let mut scopes: BTreeMap<PathBuf, BTreeSet<&str>> = BTreeMap::new();
        for rule in &config.rules {
            // Audit the declared application scope; domain-specific options can
            // select inputs rather than filter source files and must not be reinterpreted.
            let mut selected_rule = rule.clone();
            selected_rule.enabled = true;
            let selected = super::selected_rule_files(root, config, &selected_rule, files)?;
            for file in selected {
                let enabled = scopes.entry(file).or_default();
                if rule.enabled && known_rule(&rule.rule) && config.rule_configured(&rule.rule) {
                    enabled.insert(&rule.rule);
                }
            }
        }

        let comments = scopes
            .par_iter()
            .flat_map_iter(|(path, enabled)| {
                let Some(source) = super::suppression::read_directive_source(sources, path) else {
                    return Vec::new();
                };
                let file = relative_slash_path(root, path);
                let markdown = path.extension().is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown")
                });
                disable_directives(&source, !markdown)
                    .into_iter()
                    .map(|named| {
                        let status = if enabled.contains(named.rule.as_str()) {
                            DirectiveStatus::Enabled
                        } else if known_rule(&named.rule) {
                            DirectiveStatus::Disabled
                        } else {
                            DirectiveStatus::Unknown
                        };
                        PreparedComment {
                            file: file.clone(),
                            rule: named.rule,
                            directive: project(named.directive),
                            status,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok(Self {
            root: root.to_path_buf(),
            comments,
        })
    }

    /// Finalization only matches retained comments to consumed provenance.
    pub fn findings(&self, suppressed: &[SuppressedFinding]) -> Vec<RuleFinding> {
        self.findings_for_locations(suppressed.iter().map(|item| {
            (
                item.source_file.as_str(),
                item.rule.as_str(),
                &item.directive,
            )
        }))
    }

    /// Complete internal consumption, independent of deduplicated public findings.
    pub fn findings_from_consumed(
        &self,
        consumed: &[ConsumedDisableDirective],
    ) -> Vec<RuleFinding> {
        self.findings_for_locations(consumed.iter().map(|item| {
            (
                item.source_file.as_str(),
                item.rule.as_str(),
                &item.directive,
            )
        }))
    }

    fn findings_for_locations<'a>(
        &self,
        consumed: impl Iterator<Item = (&'a str, &'a str, &'a SuppressionDirective)>,
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
            if used.contains(&(comment.file.clone(), comment.rule.as_str(), comment.directive.clone())) { return None }
            let (rule, detail) = match comment.status {
                DirectiveStatus::Enabled => ("unused-disable-directive", format!("directive for `{}` suppressed no findings in this invocation", comment.rule)),
                DirectiveStatus::Disabled => ("invalid-disable-directive", format!("directive names disabled or out-of-scope rule `{}`", comment.rule)),
                DirectiveStatus::Unknown => ("invalid-disable-directive", format!("directive names unknown rule `{}`", comment.rule)),
            };
            Some(RuleFinding {
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

#[cfg(test)]
mod tests;

#[cfg(test)]
mod catalog_tests;
