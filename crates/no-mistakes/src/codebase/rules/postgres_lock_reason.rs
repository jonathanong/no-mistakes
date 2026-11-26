use super::{analysis_policy::AnalysisDiagnostics, PreparedRuleContext, RuleFinding};
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use options::{resolve_unanalyzable, CompiledOptions};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod comment;
mod comments_index;
mod embedded;
mod findings;
mod options;
mod scan;
#[cfg(test)]
mod tests;

pub const RULE_ID: &str = "postgres-lock-reason";

pub(crate) fn collect_application_errors(
    _root: &Path,
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> Result<BTreeSet<String>> {
    let mut app_errors = BTreeSet::new();
    if let Err(error) = CompiledOptions::from_rule(rule) {
        app_errors.insert(format!("{error:#}"));
    }
    if let Err(error) = resolve_unanalyzable(config, rule) {
        app_errors.insert(format!("{error:#}"));
    }
    Ok(app_errors)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    super::check_with_source_store(root, config, files, |root, config, files, sources| {
        Ok(
            diagnostics_with_files_sources_and_facts(root, config, files, sources, None, false)?
                .findings,
        )
    })
}

#[doc(hidden)]
pub fn diagnostics_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    facts: Option<&CheckFactMap>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let owned_facts;
    let facts = match facts {
        Some(prepared) => prepared,
        None => {
            owned_facts = super::prepare_sql_rule_facts(root, config, files, sources, RULE_ID)?;
            &owned_facts
        }
    };
    let context = PreparedRuleContext::new(root, config, files, sources, facts);
    let mut diagnostics = AnalysisDiagnostics::default();
    for (index, rule) in config.rule_applications(RULE_ID).into_iter().enumerate() {
        let options = CompiledOptions::from_rule(rule)?;
        let policy = resolve_unanalyzable(config, rule)?;
        let mut selected =
            super::selected_rule_files_with_sources(root, config, rule, files, sources)?;
        selected.retain(|path| options.paths.includes(&relative_slash_path(root, path)));
        selected.sort_unstable();
        selected.dedup();
        let view = facts.for_sql_application(index, rule.name.as_deref());
        let fact_ctx = view
            .as_ref()
            .map_or(facts as &dyn SqlFactContext, |view| view);
        let (mut findings, mut unanalyzable) =
            scan::collect(root, &options, &selected, sources, fact_ctx)?;
        if let Some(prefix) = &rule.message {
            for finding in findings.iter_mut().chain(unanalyzable.iter_mut()) {
                finding.message = format!("{prefix} {}", finding.message);
            }
        }
        if !defer_suppression {
            super::suppress_rule_findings_with_sources(root, &mut findings, sources);
            super::suppress_rule_findings_with_sources(root, &mut unanalyzable, sources);
        }
        diagnostics.findings.extend(findings);
        for finding in unanalyzable {
            diagnostics.unanalyzable(policy, finding);
        }
    }
    let unmatched = super::postgres_unmatched_executors::check(
        root,
        config,
        RULE_ID,
        files,
        sources,
        Some(context.facts),
    )?;
    diagnostics.findings.extend(unmatched);
    super::sort_findings(&mut diagnostics.findings);
    diagnostics.findings.dedup();
    super::sort_findings(&mut diagnostics.advisories);
    diagnostics.advisories.dedup();
    Ok(diagnostics)
}

pub(super) fn finding(file: String, line: usize, message: String, target: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file,
        line: line.max(1),
        message,
        target: Some(target),
        import: None,
        causes: Vec::new(),
    }
}
