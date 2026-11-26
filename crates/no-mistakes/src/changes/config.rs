use crate::codebase::keyed_diff::{KeyFormat, KeyOptions};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ChangesConfig {
    pub categories: BTreeMap<String, ChangeCategory>,
}

impl ChangesConfig {
    pub(crate) fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ChangeCategory {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdRule>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<ChangeRule>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<String>,
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_pattern: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub assert_pure_json_loader: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ChangeRule {
    pub paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdRule>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct IdRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys_of: Option<KeysOf>,
}

pub(crate) fn from_path_parts(template: &str) -> Option<(&str, &str)> {
    template
        .split_once("{id}")
        .filter(|(_, suffix)| !suffix.contains("{id}"))
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum KeysOf {
    Format(KeyFormat),
    Options(KeyOptions),
}

impl KeysOf {
    pub(super) fn options(&self) -> KeyOptions {
        match self {
            Self::Format(format) => KeyOptions {
                format: *format,
                array: None,
                id_field: None,
            },
            Self::Options(options) => options.clone(),
        }
    }
}
