use super::RuleFinding;
use crate::codebase::check_outcome::CheckFailure;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::vitest_lexical_policy::{Kind, Reader, Value};
use anyhow::Result;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod declared;
mod evaluate;
mod options;
use options::Options;
#[cfg(test)]
mod tests;

pub const RULE_ID: &str = "vitest-project-policy";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Parity {
    pub(crate) properties: [String; 2],
    #[serde(default)]
    pub(crate) compare_root: bool,
}

#[derive(Default)]
pub struct Diagnostics {
    pub findings: Vec<RuleFinding>,
    pub incomplete: Vec<CheckFailure>,
}

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Diagnostics> {
    crate::ast::with_request_parse_cache(|| {
        let normalized_root = crate::codebase::ts_source::normalize_discovery_path(root);
        let root = normalized_root.as_path();
        let mut output = Diagnostics::default();
        for rule in config.rule_applications(RULE_ID) {
            let options: Options = rule.try_rule_options()?;
            options.validate()?;
            let path = crate::codebase::ts_resolver::normalize_path(&root.join(&options.config));
            let (selected, _) = super::selected_rule_files_with_roots(
                root,
                config,
                rule,
                std::slice::from_ref(&path),
            )?;
            let selected = super::path_filter::filter_rule_files(root, config, rule, &selected)?;
            if !selected.contains(&path) {
                continue;
            }
            let allowed = options.source_paths(root, &path, files)?;
            let mut application = Diagnostics::default();
            if options.check_effective() {
                let mut reader = Reader::new(sources, allowed.clone());
                application = evaluate::check(root, &options, reader.read(&path));
            }
            application.findings.extend(declared::check(
                root,
                &options.declared,
                &allowed,
                sources,
            ));
            application.incomplete.retain(|failure| {
                !failure.file.as_ref().is_some_and(|file| {
                    sources.read_path(&root.join(file)).is_ok_and(|source| {
                        crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
                    })
                })
            });
            if !defer_suppression {
                super::suppress_rule_findings_with_sources(
                    root,
                    &mut application.findings,
                    sources,
                );
            }
            if let Some(message) = &rule.message {
                for finding in &mut application.findings {
                    finding.message = message.clone();
                }
            }
            output.findings.extend(application.findings);
            output.incomplete.extend(application.incomplete);
        }
        super::sort_findings(&mut output.findings);
        output.incomplete.sort();
        output.incomplete.dedup();
        Ok(output)
    })
}

pub fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, files, sources, false)
        .map(|output| output.findings)
}
pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    super::check_with_borrowed_source_store(root, config, files, check_with_files_and_sources)
}

fn finding(root: &Path, value: &Value, project: &str, message: String) -> RuleFinding {
    RuleFinding { rule: RULE_ID.into(), file: relative_slash_path(root, &value.file), line: value.line,
        message: format!("Vitest project {project}: {message}; declare literal test properties to keep project policy statically verifiable"),
        import: None, target: None }
}
fn incomplete(root: &Path, value: &Value, property: &str, project: &str) -> CheckFailure {
    CheckFailure { consumer: RULE_ID.into(), file: Some(PathBuf::from(relative_slash_path(root, &value.file))),
        property: Some(property.into()), message: format!("Vitest project {project} is incomplete: {property} is not statically known; replace dynamic configuration with literal properties in the config or declared helpers") }
}
