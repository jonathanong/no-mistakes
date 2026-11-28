use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::{CheckFailure, CheckInputFailure};
use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "dependabot-cooldown-policy";
const FILES: [&str; 2] = [".github/dependabot.yml", ".github/dependabot.yaml"];

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(in crate::codebase::rules) struct Options {
    minimum_days: Option<u64>,
    require_open_pull_requests_limit: bool,
    allowed_cooldown_excludes: Option<Vec<String>>,
    exempt_ecosystems: Vec<String>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    rules::check_with_borrowed_source_store(root, config, files, |root, config, files, sources| {
        let mut findings = check_prepared(root, config, files, sources, None)?;
        rules::suppress_rule_findings_with_sources(root, &mut findings, sources);
        Ok(findings)
    })
}

pub(crate) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    session: Option<&AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    let paths = FILES.map(|file| root.join(file));
    rules::run_rule_applications(config, RULE_ID, |rule| {
        let options: Options = rule.try_rule_options()?;
        let (minimum_days, allowed) = compile_options(&options)?;
        let selected = rules::selected_rule_files_matching_with_sources(
            root,
            config,
            rule,
            files,
            sources,
            |candidate| paths.iter().any(|path| candidate == path),
        )?;
        let mut findings = Vec::new();
        for (file, path) in FILES.into_iter().zip(&paths) {
            if !selected.contains(path) {
                continue;
            }
            let value = match read_updates(sources, path) {
                Ok(value) => value,
                Err(error) => {
                    let message = format!(
                        "Cannot verify Dependabot cooldowns: {error:#}. Repair {file} and run the check again."
                    );
                    let file = PathBuf::from(file);
                    let Some(session) = session else {
                        return Err(CheckInputFailure::source(file, message).into());
                    };
                    session.record_check_failure(CheckFailure {
                        consumer: RULE_ID.to_owned(),
                        file: Some(file),
                        property: None,
                        message,
                    });
                    continue;
                }
            };
            findings.extend(scan::check(
                file,
                value
                    .get("updates")
                    .and_then(Value::as_sequence)
                    .map(Vec::as_slice),
                sources.config_locations(path).as_deref(),
                &options,
                minimum_days,
                allowed.as_ref(),
                rule.message.as_deref(),
            ));
        }
        Ok(findings)
    })
}

pub(in crate::codebase::rules) fn compile_options(
    options: &Options,
) -> Result<(u64, Option<GlobSet>)> {
    let minimum_days = options
        .minimum_days
        .with_context(|| format!("{RULE_ID}: set options.minimumDays to a nonnegative integer"))?;
    let allowed = options
        .allowed_cooldown_excludes
        .as_deref()
        .map(compile_excludes)
        .transpose()?;
    Ok((minimum_days, allowed))
}

fn compile_excludes(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(
            GlobBuilder::new(pattern)
                .backslash_escape(true)
                .build()
                .with_context(|| {
                    format!(
                        "{RULE_ID}: invalid glob `{pattern}` in options.allowedCooldownExcludes"
                    )
                })?,
        );
    }
    builder
        .build()
        .with_context(|| format!("{RULE_ID}: could not compile options.allowedCooldownExcludes"))
}

fn read_updates(sources: &SourceStore, path: &Path) -> Result<Value> {
    let value = sources
        .parse_yaml_path(path)
        .map_err(|error| anyhow::anyhow!("could not read or parse YAML: {error}"))?;
    let mut value = value.as_ref().clone();
    crate::codebase::structured_value::expand_merge_keys(&mut value)
        .map_err(anyhow::Error::msg)
        .context("could not apply YAML merge keys")?;
    anyhow::ensure!(value.is_mapping(), "expected a top-level mapping");
    if let Some(updates) = value.get("updates") {
        let updates = updates
            .as_sequence()
            .context("expected updates to be a sequence")?;
        if let Some(index) = updates.iter().position(|update| !update.is_mapping()) {
            anyhow::bail!("expected updates[{index}] to be a mapping");
        }
    }
    Ok(value)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
