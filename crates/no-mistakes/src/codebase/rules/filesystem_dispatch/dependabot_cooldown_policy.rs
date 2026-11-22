use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::{CheckFailure, CheckInputFailure};
use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "dependabot-cooldown-policy";
const FILE: &str = ".github/dependabot.yml";

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    minimum_days: Option<u64>,
    require_open_pull_requests_limit: bool,
    allowed_cooldown_excludes: Option<Vec<String>>,
    exempt_ecosystems: Vec<String>,
}

pub(super) fn check_with_files(
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

pub(super) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    session: Option<&AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    let path = root.join(FILE);
    rules::run_rule_applications(config, RULE_ID, |rule| {
        let options: Options = rule.try_rule_options()?;
        let minimum_days = options.minimum_days.with_context(|| {
            format!("{RULE_ID}: set options.minimumDays to a nonnegative integer")
        })?;
        let allowed = options
            .allowed_cooldown_excludes
            .as_deref()
            .map(compile_excludes)
            .transpose()?;
        let selected = rules::selected_rule_files_matching_with_sources(
            root,
            config,
            rule,
            files,
            sources,
            |candidate| candidate == path,
        )?;
        if selected.is_empty() {
            return Ok(Vec::new());
        }
        match read_updates(sources, &path) {
            Ok(value) => Ok(scan::check(
                value
                    .get("updates")
                    .and_then(Value::as_sequence)
                    .map(Vec::as_slice),
                sources.config_locations(&path).as_deref(),
                &options,
                minimum_days,
                allowed.as_ref(),
                rule.message.as_deref(),
            )),
            Err(error) => {
                let message = format!(
                    "Cannot verify Dependabot cooldowns: {error:#}. Repair {FILE} and run the check again."
                );
                let file = PathBuf::from(relative_slash_path(root, &path));
                if let Some(session) = session {
                    session.record_check_failure(CheckFailure {
                        consumer: RULE_ID.to_owned(),
                        file: Some(file),
                        property: None,
                        message,
                    });
                    Ok(Vec::new())
                } else {
                    Err(CheckInputFailure::source(file, message).into())
                }
            }
        }
    })
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

fn read_updates(sources: &SourceStore, path: &Path) -> Result<std::sync::Arc<Value>> {
    let value = sources
        .parse_yaml_path(path)
        .context("could not read or parse YAML")?;
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
