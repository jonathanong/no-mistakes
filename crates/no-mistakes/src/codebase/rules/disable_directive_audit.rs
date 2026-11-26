//! Aggregate-only suppression hygiene. Findings are appended after filtering.
use super::suppression::SuppressionDirective;
use super::{RuleFinding, SuppressedFinding};
mod consumed;
mod known_rules;
mod postgres;
mod scan;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{DisableDirectivePolicy, NoMistakesConfig};
use anyhow::Result;
#[doc(hidden)]
pub use consumed::ConsumedDisableDirective;
use known_rules::known_rule;
use scan::{
    scan_scopes, DirectiveStatus, PreparedComment, INVALID_DISABLE_DIRECTIVE,
    UNUSED_DISABLE_DIRECTIVE,
};
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
    read_failures: Vec<RuleFinding>,
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
                read_failures: Vec::new(),
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

        let (comments, read_failures) = scan_scopes(root, &scopes, sources);
        Ok(Self {
            root: root.to_path_buf(),
            comments,
            read_failures,
        })
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
        let mut findings: Vec<_> = self.comments.iter().filter_map(|comment| {
            if matches!(comment.status, DirectiveStatus::Enabled) && unparsed_files.contains(&comment.file) { return None }
            if !matches!(comment.status, DirectiveStatus::IneffectivePlacement) && used.contains(&(comment.file.clone(), comment.rule.as_str(), comment.directive.clone())) { return None }
            let (rule, detail) = match comment.status {
                DirectiveStatus::Enabled => (UNUSED_DISABLE_DIRECTIVE, format!("directive for `{}` suppressed no findings in this invocation", comment.rule)),
                DirectiveStatus::Disabled => (INVALID_DISABLE_DIRECTIVE, format!("directive names disabled or out-of-scope rule `{}`", comment.rule)),
                DirectiveStatus::IneffectivePlacement => (INVALID_DISABLE_DIRECTIVE, format!("directive for `{}` in a Rust documentation comment cannot suppress findings; use a plain `//` comment instead", comment.rule)),
                DirectiveStatus::Unknown => (INVALID_DISABLE_DIRECTIVE, format!("directive names unknown rule `{}`", comment.rule)),
            };
            Some(RuleFinding { causes: Vec::new(),
                rule: rule.to_owned(), file: comment.file.clone(), line: comment.directive.line,
                message: format!("{detail}; remove or correct the comment so suppressions cannot hide stale rule intent"),
                import: None, target: Some(comment.rule.clone()),
            })
        }).collect();
        findings.extend(self.read_failures.iter().cloned());
        super::sort_findings(&mut findings);
        findings
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod catalog_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod block_doc_tests;
