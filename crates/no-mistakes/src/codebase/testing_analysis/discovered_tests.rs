use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestExecutionTarget {
    pub runner: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub workspace: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub base_command: Vec<String>,
    pub runner_args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredTests {
    pub tests: Vec<PathBuf>,
    pub targets_by_path: BTreeMap<PathBuf, Vec<TestExecutionTarget>>,
    pub used_fallback: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}
