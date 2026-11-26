use super::{
    TestPlanDependencies, TestPlanIgnoredChangedTestsFramework, TestPlanProjectDependency,
};
use serde::de::{self, value::MapAccessDeserializer, value::SeqAccessDeserializer};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A full-suite trigger that is not keyed by a dummy top-level project.
///
/// Empty `targets` is a framework-wide fallback for the listed paths (the
/// legacy path-list form). Non-empty `targets` selects only those runner
/// projects.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamedFullSuiteTrigger {
    pub name: String,
    pub paths: Vec<String>,
    #[serde(default)]
    pub targets: Vec<String>,
    /// Applies only when `targets` makes this a structured trigger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_changed_tests: Option<bool>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub superseded_by_changes: bool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct TestPlanDependenciesMap {
    #[serde(alias = "ignore_changed_tests")]
    ignore_changed_tests: Vec<TestPlanIgnoredChangedTestsFramework>,
    projects: BTreeMap<String, TestPlanProjectDependency>,
    triggers: Vec<NamedFullSuiteTrigger>,
}

impl<'de> Deserialize<'de> for TestPlanDependencies {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(DependenciesVisitor)
    }
}

struct DependenciesVisitor;

impl<'de> de::Visitor<'de> for DependenciesVisitor {
    type Value = TestPlanDependencies;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a full-suite trigger mapping or a list of named triggers")
    }

    fn visit_seq<S: de::SeqAccess<'de>>(self, sequence: S) -> Result<Self::Value, S::Error> {
        let triggers = Vec::deserialize(SeqAccessDeserializer::new(sequence))?;
        validate_named(&triggers).map_err(de::Error::custom)?;
        Ok(TestPlanDependencies {
            triggers,
            ..TestPlanDependencies::default()
        })
    }

    fn visit_map<M: de::MapAccess<'de>>(self, mapping: M) -> Result<Self::Value, M::Error> {
        let map = TestPlanDependenciesMap::deserialize(MapAccessDeserializer::new(mapping))?;
        validate_named(&map.triggers).map_err(de::Error::custom)?;
        Ok(TestPlanDependencies {
            ignore_changed_tests: map.ignore_changed_tests,
            projects: map.projects,
            triggers: map.triggers,
        })
    }
}

fn validate_named(triggers: &[NamedFullSuiteTrigger]) -> Result<(), String> {
    let mut names = BTreeMap::new();
    for (index, trigger) in triggers.iter().enumerate() {
        let name = trigger.name.trim();
        if name.is_empty() {
            return Err(format!("fullSuiteTriggers[{index}].name must not be blank"));
        }
        if trigger.paths.is_empty() {
            return Err(format!(
                "fullSuiteTriggers[{index}] `{name}` paths must not be empty"
            ));
        }
        if trigger.targets.is_empty() && trigger.include_changed_tests.is_some() {
            return Err(format!(
                "fullSuiteTriggers[{index}] `{name}` includeChangedTests requires non-empty targets"
            ));
        }
        if let Some(previous) = names.insert(name.to_string(), index) {
            return Err(format!(
                "fullSuiteTriggers[{index}] `{name}` duplicates fullSuiteTriggers[{previous}]"
            ));
        }
    }
    Ok(())
}
