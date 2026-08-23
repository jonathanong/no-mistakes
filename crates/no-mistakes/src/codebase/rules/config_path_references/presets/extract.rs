use super::super::BaseDir;
use super::oxlint;
use super::types::{is_optional_glob, Extracted};
use crate::codebase::rules::no_mistakes_config::paths::{self, Kind};
use crate::config::v2::NoMistakesConfig;
use serde_yaml::Value;

pub(crate) fn matches_preset(preset: &str, filename: &str, rel: &str) -> bool {
    match preset {
        "oxlintrc" => filename == ".oxlintrc.json" || filename == ".oxlintrc.jsonc",
        "knip" => filename == "knip.json" || filename == "knip.jsonc",
        "dependabot" => rel == ".github/dependabot.yml" || rel == ".github/dependabot.yaml",
        "sgconfig" => filename == "sgconfig.yml" || filename == "sgconfig.yaml",
        "syncpack" => filename == ".syncpackrc.json",
        "coverage-rules" => filename == ".coverage-rules.yml" || filename == ".coverage-rules.yaml",
        "pnpm-workspace-filters" => {
            (rel.starts_with(".github/workflows/")
                && (rel.ends_with(".yml") || rel.ends_with(".yaml")))
                || (rel.starts_with(".github/actions/") && rel.ends_with("/action.yml"))
                || (rel.starts_with(".github/actions/") && rel.ends_with("/action.yaml"))
        }
        "no-mistakes" => rel == ".no-mistakes.yml" || rel == ".no-mistakes.yaml",
        _ => false,
    }
}

pub(crate) fn no_mistakes(config: &NoMistakesConfig) -> Vec<Extracted> {
    paths::references(config)
        .into_iter()
        .map(|reference| Extracted {
            field: reference.field,
            value: reference.value,
            allow_globs: reference.kind == Kind::Glob,
            base_dir: BaseDir::Root,
        })
        .collect()
}

pub(crate) fn extract(preset: &str, value: &Value) -> Vec<Extracted> {
    match preset {
        "oxlintrc" => oxlint::extract(value),
        "knip" => knip(value),
        "dependabot" => dependabot(value),
        "sgconfig" => strings_at(value, "ruleDirs", "ruleDirs", false, BaseDir::ConfigFile),
        "syncpack" => strings_at(value, "source", "source", true, BaseDir::Root),
        "coverage-rules" => coverage_rules(value),
        _ => Vec::new(),
    }
}

fn strings(value: &Value) -> Vec<String> {
    match value {
        Value::String(value) => vec![value.clone()],
        Value::Sequence(values) => values
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn strings_at(
    value: &Value,
    key: &str,
    field: &str,
    allow_globs: bool,
    base_dir: BaseDir,
) -> Vec<Extracted> {
    strings(value.get(key).unwrap_or(&Value::Null))
        .into_iter()
        .enumerate()
        .map(|(index, path)| Extracted {
            field: format!("{field}[{index}]"),
            value: path,
            allow_globs,
            base_dir,
        })
        .collect()
}

fn dependabot(value: &Value) -> Vec<Extracted> {
    let Some(updates) = value.get("updates").and_then(Value::as_sequence) else {
        return Vec::new();
    };
    updates
        .iter()
        .enumerate()
        .filter_map(|(index, update)| {
            let directory = update.get("directory")?.as_str()?;
            Some(Extracted {
                field: format!("updates[{index}].directory"),
                value: normalize_root_directory(directory),
                allow_globs: false,
                base_dir: BaseDir::Root,
            })
        })
        .collect()
}

fn normalize_root_directory(directory: &str) -> String {
    let trimmed = directory.trim_start_matches('/');
    if trimmed.is_empty() {
        ".".to_string()
    } else {
        trimmed.to_string()
    }
}

fn coverage_rules(value: &Value) -> Vec<Extracted> {
    let Some(rules) = value.get("rules").and_then(Value::as_sequence) else {
        return Vec::new();
    };
    rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| {
            let paths = rule.get("paths")?.as_str()?;
            if is_optional_glob(paths) {
                return None;
            }
            Some(Extracted {
                field: format!("rules[{index}].paths"),
                value: paths.to_string(),
                allow_globs: true,
                base_dir: BaseDir::Root,
            })
        })
        .collect()
}

fn knip(value: &Value) -> Vec<Extracted> {
    let Some(workspaces) = value.get("workspaces").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    let mut extracted = Vec::new();
    for (workspace, config) in workspaces {
        let Some(workspace) = workspace.as_str() else {
            continue;
        };
        knip_workspace(&mut extracted, workspace, config);
    }
    extracted
}

fn knip_workspace(extracted: &mut Vec<Extracted>, workspace: &str, config: &Value) {
    let prefix = if workspace == "." {
        String::new()
    } else {
        format!("{workspace}/")
    };
    for key in ["entry", "project"] {
        for (index, path) in strings(config.get(key).unwrap_or(&Value::Null))
            .into_iter()
            .enumerate()
        {
            if is_optional_glob(&path) {
                continue;
            }
            extracted.push(Extracted {
                field: format!("workspaces.{workspace}.{key}[{index}]"),
                value: format!("{prefix}{path}"),
                allow_globs: true,
                base_dir: BaseDir::Root,
            });
        }
    }
}
