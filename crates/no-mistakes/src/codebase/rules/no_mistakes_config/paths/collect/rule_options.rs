use super::{path_kind, push, Kind, Ref};
use crate::config::v2::NoMistakesConfig;
use serde_yaml::Value;

pub(super) fn collect(config: &NoMistakesConfig, refs: &mut Vec<Ref>) {
    for (index, rule) in config.rules.iter().enumerate() {
        collect_paths(&rule.options, &format!("rules[{index}].options"), refs);
    }
}

fn collect_paths(value: &Value, field: &str, refs: &mut Vec<Ref>) {
    let Some(map) = value.as_mapping() else {
        return;
    };
    for (key, value) in map {
        let Some(key) = key.as_str() else {
            continue;
        };
        let child = format!("{field}.{key}");
        match key {
            "tsconfig" | "lockfile" | "shellFiles" | "allowlist" => {
                for (index, path) in string_values(value).into_iter().enumerate() {
                    if let Some(path) = required_path(&path) {
                        push(refs, format!("{child}[{index}]"), Kind::File, &path);
                    }
                }
            }
            "roots" | "selectorRoots" | "shebangDirs" => {
                for (index, path) in string_values(value).into_iter().enumerate() {
                    if let Some(path) = required_path(&path) {
                        push(
                            refs,
                            format!("{child}[{index}]"),
                            if source_file_path(&path) {
                                Kind::File
                            } else {
                                Kind::Directory
                            },
                            &path,
                        );
                    }
                }
            }
            "packages" => collect_packages(value, &child, refs),
            // Exclusions are deliberately not validated: a defensive exclude may
            // refer to a path that is absent until a later feature is introduced.
            "excludePaths" | "conditionallyAllowedWorkflows" => {}
            _ => collect_paths(value, &child, refs),
        }
    }
}

fn collect_packages(value: &Value, field: &str, refs: &mut Vec<Ref>) {
    let Some(packages) = value.as_sequence() else {
        return;
    };
    for (index, package) in packages.iter().enumerate() {
        let Some(root) = package.get("root").and_then(Value::as_str) else {
            continue;
        };
        if let Some(root) = required_path(root) {
            push(
                refs,
                format!("{field}[{index}].root"),
                Kind::Directory,
                &root,
            );
        }
    }
}

fn string_values(value: &Value) -> Vec<String> {
    match value {
        Value::String(value) => vec![value.clone()],
        Value::Sequence(values) => values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn required_path(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && !value.starts_with('!') && !path_kind(value).eq(&Kind::Glob))
        .then(|| value.to_string())
}

fn source_file_path(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        ".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".mts", ".cts", ".json",
    ]
    .iter()
    .any(|suffix| value.ends_with(suffix))
}
