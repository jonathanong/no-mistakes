use super::{
    TestPlanDependencies, TestPlanIgnoredChangedTestsFramework, TestPlanProjectDependency,
};
use serde::de::{self, Deserializer};
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
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct TestPlanDependenciesMap {
    #[serde(alias = "ignore_changed_tests")]
    ignore_changed_tests: Vec<TestPlanIgnoredChangedTestsFramework>,
    projects: BTreeMap<String, TestPlanProjectDependency>,
    triggers: Vec<NamedFullSuiteTrigger>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TestPlanDependenciesRaw {
    List(Vec<NamedFullSuiteTrigger>),
    Map(TestPlanDependenciesMap),
}

impl<'de> Deserialize<'de> for TestPlanDependencies {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match TestPlanDependenciesRaw::deserialize(deserializer)? {
            TestPlanDependenciesRaw::List(triggers) => {
                validate_named(&triggers).map_err(de::Error::custom)?;
                Ok(TestPlanDependencies {
                    triggers,
                    ..TestPlanDependencies::default()
                })
            }
            TestPlanDependenciesRaw::Map(map) => {
                validate_named(&map.triggers).map_err(de::Error::custom)?;
                Ok(TestPlanDependencies {
                    ignore_changed_tests: map.ignore_changed_tests,
                    projects: map.projects,
                    triggers: map.triggers,
                })
            }
        }
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
