use super::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::rules::path_filter::GlobMatcher;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) mod ast;
mod findings;
mod options;
pub(crate) mod playwright_provenance;
mod scope;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
mod workflow;

use findings::{findings_for, source_findings, unreadable, RuleContext};
use options::{Allowlist, RetryOptions, SerialOptions};
use scope::Origin;
pub use scope::{required_files, SourceDemand};

pub const RULE_ID: &str = "test-no-retries";
pub use RULE_ID as TEST_NO_RETRIES;
pub const PLAYWRIGHT_NO_SERIAL: &str = "playwright-no-serial";

pub(crate) fn configured(config: &NoMistakesConfig) -> bool {
    super::rule_enabled(config, TEST_NO_RETRIES)
        || super::rule_enabled(config, PLAYWRIGHT_NO_SERIAL)
}

pub(crate) struct Location {
    shape: Shape,
    line: usize,
    project: Option<String>,
    origin: Origin,
    pub(crate) playwright_import: Option<playwright_provenance::ImportRef>,
    pub(crate) playwright_binding: Option<Arc<str>>,
    pub(crate) playwright_wrappers: Vec<playwright_provenance::WrapperCall>,
    pub(crate) resolved_playwright_exports: Vec<(String, Arc<str>)>,
    pub(crate) unresolved_reason: Option<&'static str>,
}

enum Shape {
    Retry(&'static str, Option<u64>),
    Serial(&'static str),
}

pub(crate) struct Inputs<'a> {
    pub(crate) root: &'a Path,
    pub(crate) files: &'a [PathBuf],
    pub(crate) sources: &'a SourceStore,
    pub(crate) facts: Option<&'a CheckFactMap>,
    pub(crate) session: Option<&'a AnalysisSession>,
}

pub(crate) fn check_prepared(
    rule_id: &str,
    config: &NoMistakesConfig,
    inputs: Inputs<'_>,
) -> Result<Vec<RuleFinding>> {
    let paths = scope::ConfigPaths::new(inputs.root, config)?;
    super::run_rule_applications(config, rule_id, |rule| {
        let (max_retries, allowlist, test_commands, factory_files, fixtures) =
            if rule_id == TEST_NO_RETRIES {
                let options: RetryOptions = rule.try_rule_options()?;
                options::validate_test_commands(&options.test_commands)?;
                let factory_files = options.factory_files()?;
                (
                    options.max_retries,
                    options.allowlist,
                    options.test_commands,
                    factory_files,
                    options.fixtures,
                )
            } else {
                let options: SerialOptions = rule.try_rule_options()?;
                (
                    0,
                    options.allowlist,
                    Vec::new(),
                    GlobMatcher::new(&[], rule_id)?,
                    options.fixtures,
                )
            };
        let allowlist = Allowlist::compile(rule_id, allowlist)?;
        let fixtures = options::compile_fixtures(rule_id, fixtures)?;
        let ctx = RuleContext {
            id: rule_id,
            max_retries,
            allowlist: &allowlist,
            fixtures: &fixtures,
        };
        let files = super::selected_rule_files_with_sources(
            inputs.root,
            config,
            rule,
            inputs.files,
            inputs.sources,
        )?;
        let findings: Result<Vec<_>> = files
            .par_iter()
            .map(|path| {
                let source = paths.source(inputs.root, path, rule, &factory_files);
                let workflow = rule_id == TEST_NO_RETRIES && paths.workflow(path);
                if source.is_none() && !workflow {
                    return Ok(None);
                }
                let relative = relative_slash_path(inputs.root, path);
                if allowlist.paths.is_match(&relative) {
                    return Ok(None);
                }
                let result = if let Some(source) = source {
                    source_findings(path, &relative, &inputs, &ctx, source)
                } else {
                    workflow::collect(path, inputs.sources, &test_commands)
                        .map(|locations| {
                            findings_for(
                                &relative,
                                rule_id,
                                max_retries,
                                &allowlist,
                                locations.iter(),
                            )
                        })
                        .or_else(|error| {
                            unreadable(&inputs, path, &relative, rule_id, error.to_string())
                        })
                };
                if let Some(session) = inputs.session {
                    session.capture_check_result(rule_id, result)
                } else {
                    result.map(Some)
                }
            })
            .collect();
        let mut findings: Vec<_> = findings?.into_iter().flatten().flatten().collect();
        if let Some(message) = &rule.message {
            for finding in &mut findings {
                finding.message.clone_from(message);
            }
        }
        Ok(findings)
    })
}
