use super::super::BaseDir;
use super::oxlint;
use super::types::{is_optional_glob, Extracted};
use regex::Regex;
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
        _ => false,
    }
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

pub(crate) fn pnpm_workspace_filters(source: &str) -> Vec<Extracted> {
    let filter = Regex::new(r#"--filter\s+(?:"([^"]+)"|'([^']+)'|([^\s'"\\]+))"#)
        .expect("pnpm filter pattern is valid");
    let mut extracted = Vec::new();
    let mut index = 0;
    for capture in filter.captures_iter(source) {
        let raw = capture
            .get(1)
            .or_else(|| capture.get(2))
            .or_else(|| capture.get(3))
            .map(|match_| match_.as_str())
            .unwrap_or_default();
        let Some(path) = normalize_pnpm_filter(raw) else {
            continue;
        };
        if is_pnpm_filter_guarded(source, &path) || path.starts_with('!') {
            continue;
        }
        extracted.push(Extracted {
            field: format!("pnpm filter {index}"),
            allow_globs: path.contains('*') || path.contains('?') || path.contains('{'),
            base_dir: BaseDir::Root,
            value: path,
        });
        index += 1;
    }
    extracted
}

fn normalize_pnpm_filter(raw: &str) -> Option<String> {
    let mut path = raw.trim();
    if let Some(braced) = path.strip_prefix('{') {
        path = braced.strip_suffix("}...").unwrap_or(braced);
        path = path.strip_suffix('}').unwrap_or(path);
    } else {
        path = path.strip_suffix("...").unwrap_or(path);
    }
    if !path.starts_with("./") || path == "./" {
        return None;
    }
    Some(path.to_string())
}

fn is_pnpm_filter_guarded(source: &str, path: &str) -> bool {
    if path.contains('*') || path.contains('?') || path.contains('{') {
        return false;
    }
    let normalized = path.trim_start_matches("./");
    let escaped = regex::escape(normalized);
    let directory = format!(r#"(?:\./)?{escaped}"#);
    let file = format!(r#"(?:\./)?{escaped}(?:/package\.json)?"#);
    let patterns = [
        format!(r#"\[\s+-f\s+[\"']?{file}[\"']?\s*\]"#),
        format!(r#"\[\s+-d\s+[\"']?{directory}[\"']?\s*\]"#),
        format!(r#"test\s+-f\s+[\"']?{file}[\"']?"#),
        format!(r#"test\s+-d\s+[\"']?{directory}[\"']?"#),
    ];
    patterns.iter().any(|pattern| {
        Regex::new(pattern)
            .expect("pnpm filter guard pattern is valid")
            .is_match(source)
    })
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
