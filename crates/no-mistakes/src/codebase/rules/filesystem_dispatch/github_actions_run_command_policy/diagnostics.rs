use super::options::{self, Options};
use super::runs;
use super::RULE_ID;
use crate::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(super) fn run(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        let compiled = options::compile(options)?;
        let (selected, roots) = rules::selected_rule_files_with_roots(root, config, rule, files)?;
        let selected = if compiled.include.is_empty() {
            selected
        } else {
            rules::matching_files(root, &compiled.include, &selected, &roots)?
        };
        let unanalyzable = resolve_unanalyzable(config, rule)?;
        if compiled.commands.is_empty() {
            continue;
        }
        let chunks: Vec<_> = selected
            .par_iter()
            .filter_map(|path| {
                let rel = relative_slash_path(root, path);
                let action = rules::github_actions_workflow_files::gha_kind(&rel)?;
                let content = rules::read_language_source(sources, path)?;
                Some(runs::check_source(
                    &rel,
                    &content,
                    action,
                    &compiled,
                    unanalyzable,
                    rule.message.as_deref(),
                ))
            })
            .collect();
        for chunk in chunks {
            output.append(chunk);
        }
    }
    if !defer_suppression {
        rules::suppress_rule_findings_with_sources(root, &mut output.findings, sources);
        rules::suppress_rule_findings_with_sources(root, &mut output.advisories, sources);
    }
    rules::sort_findings(&mut output.findings);
    rules::sort_findings(&mut output.advisories);
    Ok(output)
}

fn resolve_unanalyzable(
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> Result<Unanalyzable> {
    let explicit = rule.options.get("unanalyzable").is_some() || config.unanalyzable.is_some();
    if explicit {
        Ok(config.effective_rule_analysis(rule)?.unanalyzable)
    } else {
        Ok(Unanalyzable::Advise)
    }
}
