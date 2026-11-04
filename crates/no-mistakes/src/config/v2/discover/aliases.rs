use crate::config::source_locations::{ConfigPath, Segment};
use anyhow::Result;
use serde_yaml::{Mapping, Value};
use std::path::Path;

pub(super) fn canonical_key(parent: &ConfigPath, key: &str) -> String {
    let parts = parent.as_slice();
    let plan = matches!(parts.first(), Some(Segment::Key(name)) if name == "testPlan");
    let replacement = match key {
        "test_plan" if parts.is_empty() => "testPlan",
        "dependencies" if plan && parts.len() == 2 => "fullSuiteTriggers",
        "ignore_changed_tests"
            if plan
                && parts.len() == 3
                && matches!(parts.last(), Some(Segment::Key(name)) if name == "fullSuiteTriggers") =>
        {
            "ignoreChangedTests"
        }
        "global_config_fallback"
            if plan
                && parts.len() == 4
                && matches!(parts.get(2), Some(Segment::Key(name)) if name == "environments") =>
        {
            "globalConfigFallback"
        }
        "sample_when_limited"
            if plan
                && parts.len() == 6
                && matches!(parts.get(4), Some(Segment::Key(name)) if name == "groups") =>
        {
            "sampleWhenLimited"
        }
        _ => key,
    };
    replacement.to_string()
}

pub(super) fn normalize(value: Value, global: &ConfigPath, file: &Path) -> Result<Value> {
    Ok(match value {
        Value::Mapping(mapping) => {
            let mut result = Mapping::new();
            for (key, value) in mapping {
                let (key, child) = if let Some(raw) = key.as_str() {
                    let name = canonical_key(global, raw);
                    (Value::String(name.clone()), global.clone().key(name))
                } else {
                    (key, global.clone())
                };
                let value = normalize(value, &child, file)?;
                if result.insert(key.clone(), value).is_some() {
                    anyhow::bail!(
                        "conflicting config key aliases {key:?} in {} and {}",
                        file.display(),
                        file.display()
                    );
                }
            }
            Value::Mapping(result)
        }
        Value::Sequence(items) => Value::Sequence(
            items
                .into_iter()
                .enumerate()
                .map(|(index, item)| normalize(item, &global.clone().index(index), file))
                .collect::<Result<_>>()?,
        ),
        value => value,
    })
}

pub(super) fn deprecated_frameworks(value: &Value, section: Option<&str>) -> Vec<String> {
    let plan = match section {
        Some("testPlan" | "test_plan") => Some(value),
        None => value.get("testPlan").or_else(|| value.get("test_plan")),
        _ => None,
    };
    plan.and_then(Value::as_mapping)
        .into_iter()
        .flat_map(|mapping| mapping.iter())
        .filter_map(|(name, framework)| {
            framework
                .get("dependencies")
                .and(name.as_str())
                .map(str::to_string)
        })
        .collect()
}

pub(super) fn mark_deprecated(config: &mut super::NoMistakesConfig, frameworks: &[String]) {
    for name in frameworks {
        let plan = match name.as_str() {
            "dotnet" => &mut config.test_plan.dotnet,
            "playwright" => &mut config.test_plan.playwright,
            "vitest" => &mut config.test_plan.vitest,
            "swift" => &mut config.test_plan.swift,
            "python" => &mut config.test_plan.python,
            "go" => &mut config.test_plan.go,
            "cargo" => &mut config.test_plan.cargo,
            "rails" => &mut config.test_plan.rails,
            "php" => &mut config.test_plan.php,
            "java" => &mut config.test_plan.java,
            "kotlin" => &mut config.test_plan.kotlin,
            "elixir" => &mut config.test_plan.elixir,
            "dart" => &mut config.test_plan.dart,
            "jest" => &mut config.test_plan.jest,
            _ => continue,
        };
        plan.deprecated_dependencies_key = true;
    }
}

pub(super) fn wrap_section(value: Value, section: Option<&str>) -> Value {
    if let Some(section) = section {
        let mut mapping = Mapping::new();
        mapping.insert(
            Value::String(canonical_key(&ConfigPath::default(), section)),
            value,
        );
        Value::Mapping(mapping)
    } else {
        value
    }
}
