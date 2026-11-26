use super::file_diagnostics::{self, InputUncertainty};
use super::{analysis_policy::AnalysisDiagnostics, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::source_locations::ConfigPath;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "package-json-export-target-prefix";

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::codebase::rules) struct Options {}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::check_sourced_standalone(root, config, check_with_files)
}

super::diagnostic_source_rule_entrypoints!();

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        rule.try_rule_options::<Options>()?;
        let manifests = super::selected_rule_files_matching_with_sources(
            root,
            config,
            rule,
            files,
            sources,
            |path| path.file_name().is_some_and(|name| name == "package.json"),
        )?;
        let findings = manifests
            .par_iter()
            .flat_map(|path| check_manifest(root, path, sources))
            .collect();
        output.append(file_diagnostics::application(
            root,
            sources,
            config,
            rule,
            findings,
            defer_suppression,
            InputUncertainty::ReadOrParse,
        )?);
    }
    Ok(file_diagnostics::sorted(output))
}

fn check_manifest(root: &Path, path: &Path, sources: &SourceStore) -> Vec<RuleFinding> {
    let file = relative_slash_path(root, path);
    let manifest = match sources.parse_json_path(path) {
        Ok(manifest) => manifest,
        Err(error) => {
            return vec![finding(&file, 1, "unanalyzable".into(), format!(
                "{file}: cannot verify package export targets ({error}). Repair the JSON or restore read access."
            ))];
        }
    };
    let Some(manifest) = manifest.as_object() else {
        return vec![finding(
            &file,
            1,
            "unanalyzable".into(),
            format!("{file}: cannot verify package export targets. Replace the top-level JSON value with a package manifest object."),
        )];
    };
    let Some(exports) = manifest.get("exports") else {
        return Vec::new();
    };
    let mut invalid = Vec::new();
    collect_invalid_targets(
        exports,
        ConfigPath::default().key("exports"),
        "/exports".into(),
        &mut invalid,
    );
    if invalid.is_empty() {
        return Vec::new();
    }
    let locations = sources.config_locations(path);
    invalid
        .into_iter()
        .map(|(location, pointer, defect)| {
            let line = locations
                .as_ref()
                .and_then(|locations| locations.line(&location))
                .unwrap_or(1);
            let message = match defect {
                TargetDefect::Prefix => format!(
                    "{file}:{line}: Node.js requires the target at {pointer} to start with \"./\". Use a path relative to the package root."
                ),
                TargetDefect::Type => format!(
                    "{file}:{line}: Node.js does not accept a boolean or number at {pointer}. Use a package-relative string or null to block the export."
                ),
            };
            finding(&file, line, pointer, message)
        })
        .collect()
}

enum TargetDefect {
    Prefix,
    Type,
}

fn collect_invalid_targets(
    target: &Value,
    location: ConfigPath,
    pointer: String,
    invalid: &mut Vec<(ConfigPath, String, TargetDefect)>,
) {
    match target {
        Value::String(value) if !value.starts_with("./") => {
            invalid.push((location, pointer, TargetDefect::Prefix));
        }
        Value::Bool(_) | Value::Number(_) => {
            invalid.push((location, pointer, TargetDefect::Type));
        }
        Value::Object(entries) => {
            for (key, value) in entries {
                let escaped = key.replace('~', "~0").replace('/', "~1");
                collect_invalid_targets(
                    value,
                    location.clone().key(key),
                    format!("{pointer}/{escaped}"),
                    invalid,
                );
            }
        }
        Value::String(_) | Value::Null | Value::Array(_) => {}
    }
}

fn finding(file: &str, line: usize, target: String, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message,
        import: None,
        target: Some(target),
        causes: Vec::new(),
    }
}
