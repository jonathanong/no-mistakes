use super::RuleFinding;
use crate::codebase::check_outcome::CheckFailure;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::vitest_lexical_policy::{Kind, Reader, Value};
use anyhow::Result;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod evaluate;
#[cfg(test)]
mod tests;

pub const RULE_ID: &str = "vitest-project-policy";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Options {
    pub(crate) config: String,
    pub(crate) helpers: Vec<String>,
    pub(crate) required: BTreeMap<String, serde_json::Value>,
    pub(crate) forbidden: Vec<String>,
    pub(crate) parity: Vec<Parity>,
}
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
    _files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Diagnostics> {
    let normalized_root = crate::codebase::ts_source::normalize_discovery_path(root);
    let root = normalized_root.as_path();
    let mut output = Diagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        validate(&options)?;
        let path = crate::codebase::ts_resolver::normalize_path(&root.join(&options.config));
        let (selected, _) =
            super::selected_rule_files_with_roots(root, config, rule, std::slice::from_ref(&path))?;
        let selected = super::path_filter::filter_rule_files(root, config, rule, &selected)?;
        if !selected.contains(&path) {
            continue;
        }
        let allowed = std::iter::once(path.clone())
            .chain(
                options
                    .helpers
                    .iter()
                    .map(|helper| crate::codebase::ts_resolver::normalize_path(&root.join(helper))),
            )
            .collect::<BTreeSet<_>>();
        let mut reader = Reader::new(sources, allowed);
        let value = reader.read(&path);
        let mut application = evaluate::check(root, &options, value);
        application.incomplete.retain(|failure| {
            !failure.file.as_ref().is_some_and(|file| {
                sources.read_path(&root.join(file)).is_ok_and(|source| {
                    crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
                })
            })
        });
        if !defer_suppression {
            super::suppress_rule_findings_with_sources(root, &mut application.findings, sources);
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

fn validate(options: &Options) -> Result<()> {
    anyhow::ensure!(
        !options.config.is_empty(),
        "{RULE_ID} requires options.config naming an explicit config file"
    );
    anyhow::ensure!(
        options
            .required
            .values()
            .all(|value| !value.is_array() && !value.is_object()),
        "{RULE_ID} required values must be scalar JSON literals"
    );
    anyhow::ensure!(
        options
            .required
            .keys()
            .chain(options.forbidden.iter())
            .chain(
                options
                    .parity
                    .iter()
                    .flat_map(|pair| pair.properties.iter())
            )
            .all(|key| !key.is_empty() && !key.contains('.')),
        "{RULE_ID} property names must be nonempty direct test property names"
    );
    Ok(())
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
