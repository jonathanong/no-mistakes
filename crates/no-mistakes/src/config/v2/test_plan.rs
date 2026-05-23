use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanConfig {
    pub playwright: TestPlanFrameworkConfig,
    pub vitest: TestPlanFrameworkConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanFrameworkConfig {
    pub dependencies: TestPlanDependencies,
    pub environments: BTreeMap<String, TestPlanEnvironment>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanDependencies {
    pub projects: BTreeMap<String, TestPlanProjectDependency>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum TestPlanProjectDependency {
    All(bool),
    Patterns(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanEnvironment {
    pub all: bool,
    pub limit: Option<TestPlanLimit>,
    pub groups: Vec<TestPlanGroup>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanGroup {
    #[serde(rename = "type")]
    pub type_: TestPlanGroupType,
    pub limit: Option<TestPlanLimit>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TestPlanGroupType {
    #[default]
    Direct,
    Coverage,
    Dependencies,
    Sample,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanLimit {
    pub percent: Option<TestPlanPercent>,
    pub files: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum TestPlanPercent {
    Number(f64),
    String(String),
}

impl TestPlanPercent {
    pub fn value(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::String(raw) => raw.trim().trim_end_matches('%').parse().ok(),
        }
    }
}
