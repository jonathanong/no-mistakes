use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::StringOrList;

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Tests {
    pub playwright: PlaywrightTestConfig,
    pub vitest: VitestConfig,
    pub jest: JestConfig,
    pub storybook: StorybookConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PlaywrightTestConfig {
    pub configs: Option<StringOrList>,
    pub projects: BTreeMap<String, TestProjectPolicy>,
    pub selectors: PlaywrightSelectors,
    pub selector_roots: Vec<String>,
    pub selector_exclude: Vec<String>,
    pub frontend_root: Option<String>,
    pub ignore_routes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PlaywrightSelectors {
    pub html_ids: bool,
    pub test_ids: Vec<String>,
    pub component_test_ids: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct VitestConfig {
    pub configs: Option<StringOrList>,
    pub projects: BTreeMap<String, TestProjectPolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct JestConfig {
    pub configs: Option<StringOrList>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct StorybookConfig {
    pub configs: Option<StringOrList>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestProjectPolicy {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    #[serde(rename = "integration_suites")]
    pub integration_suites: BTreeMap<String, Vec<String>>,
}
