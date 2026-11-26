use super::{StringOrList, TestProjectPolicy};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PythonConfig {
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct GoConfig {
    pub modules: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RustLangConfig {
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RailsConfig {
    pub apps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PhpConfig {
    pub framework: Option<String>,
    pub apps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct DotnetConfig {
    pub solutions: Vec<String>,
    pub projects: BTreeMap<String, DotnetProjectConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DotnetProjectConfig {
    pub project: String,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub test: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct JestConfig {
    pub configs: Option<StringOrList>,
    pub projects: BTreeMap<String, TestProjectPolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct StorybookConfig {
    pub configs: Option<StringOrList>,
}
