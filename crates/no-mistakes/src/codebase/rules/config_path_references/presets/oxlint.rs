use super::super::BaseDir;
use super::types::{is_optional_glob, Extracted};
use serde_yaml::Value;

pub(super) fn extract(value: &Value) -> Vec<Extracted> {
    let mut extracted = Vec::new();
    plugins(value, &mut extracted);
    overrides(value, &mut extracted);
    baselines(value, &mut extracted);
    extracted
}

fn plugins(value: &Value, extracted: &mut Vec<Extracted>) {
    let Some(plugins) = value.get("jsPlugins").and_then(Value::as_sequence) else {
        return;
    };
    for (index, plugin) in plugins.iter().enumerate() {
        let Some(specifier) = plugin.get("specifier").and_then(Value::as_str) else {
            continue;
        };
        if !specifier.starts_with('.') {
            continue;
        }
        extracted.push(Extracted {
            field: format!("jsPlugins[{index}].specifier"),
            value: specifier.to_string(),
            allow_globs: false,
            base_dir: BaseDir::ConfigFile,
        });
    }
}

fn overrides(value: &Value, extracted: &mut Vec<Extracted>) {
    let Some(overrides) = value.get("overrides").and_then(Value::as_sequence) else {
        return;
    };
    for (override_index, override_value) in overrides.iter().enumerate() {
        let Some(files) = override_value.get("files").and_then(Value::as_sequence) else {
            continue;
        };
        for (file_index, file) in files.iter().enumerate() {
            let Some(path) = file.as_str() else {
                continue;
            };
            if super::super::references::has_glob_metachar(path) {
                continue;
            }
            extracted.push(Extracted {
                field: format!("overrides[{override_index}].files[{file_index}]"),
                value: path.to_string(),
                allow_globs: false,
                base_dir: BaseDir::ConfigFile,
            });
        }
    }
}

fn baselines(value: &Value, extracted: &mut Vec<Extracted>) {
    let Some(rules) = value.get("rules").and_then(Value::as_mapping) else {
        return;
    };
    for (name, config) in rules {
        let Some(name) = name.as_str() else {
            continue;
        };
        let Some(options) = config.as_sequence().and_then(|seq| seq.get(1)) else {
            continue;
        };
        let Some(baseline) = options.get("baseline").and_then(Value::as_sequence) else {
            continue;
        };
        for (index, entry) in baseline.iter().enumerate() {
            let Some(path) = entry
                .as_sequence()
                .and_then(|row| row.first())
                .and_then(Value::as_str)
            else {
                continue;
            };
            if is_optional_glob(path) {
                continue;
            }
            extracted.push(Extracted {
                field: format!("rules.{name}.baseline[{index}][0]"),
                value: path.to_string(),
                allow_globs: false,
                base_dir: BaseDir::Root,
            });
        }
    }
}
