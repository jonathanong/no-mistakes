use super::options::{self, Options};
use super::runs;
use super::RULE_ID;
use crate::codebase::rules::{self, source_access, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use rayon::prelude::*;
use serde_yaml::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;

enum ScanTarget<'a> {
    Workflow(&'a Path, bool),
    Shell(&'a Path),
}

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
        let unanalyzable = resolve_unanalyzable(config, rule)?;
        if compiled.commands.is_empty() {
            continue;
        }
        let workflow_files = if compiled.include.is_empty() {
            selected.clone()
        } else {
            rules::matching_files(root, &compiled.include, &selected, &roots)?
        };
        let shell_files = if compiled.shell_files.is_empty() {
            Vec::new()
        } else {
            rules::matching_files(root, &compiled.shell_files, &selected, &roots)?
        };
        let mut targets: Vec<ScanTarget<'_>> = Vec::new();
        for path in &workflow_files {
            let rel = relative_slash_path(root, path);
            if let Some(action) = rules::github_actions_workflow_files::gha_kind(&rel) {
                targets.push(ScanTarget::Workflow(path, action));
            }
        }
        for path in &shell_files {
            let rel = relative_slash_path(root, path);
            if rules::github_actions_workflow_files::gha_kind(&rel).is_none() {
                targets.push(ScanTarget::Shell(path));
            }
        }
        let chunks: Vec<_> = targets
            .par_iter()
            .map(|target| {
                let result = match target {
                    ScanTarget::Workflow(path, action) => {
                        let rel = relative_slash_path(root, path);
                        load_file(root, path, sources).map(|(source, value)| {
                            runs::check_parsed(
                                &rel,
                                source.trim_start_matches('\u{feff}'),
                                *action,
                                &value,
                                &compiled,
                                unanalyzable,
                                rule.message.as_deref(),
                            )
                        })
                    }
                    ScanTarget::Shell(path) => {
                        let rel = relative_slash_path(root, path);
                        read_source(root, path, sources).map(|source| {
                            runs::check_shell_source(
                                &rel,
                                &source,
                                &compiled,
                                unanalyzable,
                                rule.message.as_deref(),
                            )
                        })
                    }
                };
                match result {
                    Ok(diagnostics) => diagnostics,
                    Err(finding) => {
                        let mut diagnostics = AnalysisDiagnostics::default();
                        diagnostics.unanalyzable(unanalyzable, *finding);
                        diagnostics
                    }
                }
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

fn read_source(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
) -> Result<Arc<str>, Box<RuleFinding>> {
    sources
        .read_source_path(path)
        .map_err(|error| Box::new(source_access::read_failure(root, path, RULE_ID, &error)))
}

fn load_file(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
) -> Result<(Arc<str>, Arc<Value>), Box<RuleFinding>> {
    let source = read_source(root, path, sources)?;
    let value = sources
        .parse_loaded_yaml(path, &source, true)
        .map_err(|error| {
            let mut finding = source_access::unanalyzable_source(
                root,
                path,
                RULE_ID,
                format_args!("YAML could not be parsed ({error})"),
            );
            finding.line = error.line();
            Box::new(finding)
        })?;
    Ok((source, value))
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
