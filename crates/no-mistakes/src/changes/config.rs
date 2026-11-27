use crate::codebase::keyed_diff::{KeyFormat, KeyOptions};
use anyhow::{anyhow, ensure, Result};
use globset::{Glob, GlobBuilder};
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

pub(crate) enum IdSource<'a> {
    FromPath {
        prefix: &'a str,
        suffix: &'a str,
        separator: Option<&'a str>,
    },
    Keys(KeyOptions),
}

pub(crate) struct IdRuleError {
    pub(crate) field: Option<&'static str>,
    pub(crate) error: anyhow::Error,
}

impl IdRule {
    pub(crate) fn source(&self) -> Result<IdSource<'_>, IdRuleError> {
        match (self.from_path.as_deref(), self.keys_of.as_ref()) {
            (Some(template), None) => {
                let (prefix, suffix) = template
                    .split_once("{id}")
                    .filter(|(_, suffix)| !suffix.contains("{id}"))
                    .ok_or_else(|| IdRuleError {
                        field: Some("fromPath"),
                        error: anyhow!("`{template}` requires exactly one {{id}} placeholder"),
                    })?;
                Ok(IdSource::FromPath {
                    prefix,
                    suffix,
                    separator: self.separator.as_deref(),
                })
            }
            (None, Some(keys)) if self.separator.is_none() => {
                let options = keys.options();
                options.validate().map_err(|error| IdRuleError {
                    field: Some("keysOf"),
                    error,
                })?;
                Ok(IdSource::Keys(options))
            }
            _ => Err(IdRuleError {
                field: None,
                error: anyhow!(
                    "declare either fromPath or keysOf, with separator only for fromPath"
                ),
            }),
        }
    }
}

pub(crate) fn validate_category_name(name: &str) -> Result<()> {
    ensure!(
        !name.is_empty()
            && !matches!(
                name,
                "schemaVersion" | "complete" | "unclassified" | "changedFiles" | "reasons"
            ),
        "invalid changes category name {name:?}"
    );
    Ok(())
}

pub(crate) fn compile_selection_glob(pattern: &str) -> Result<Glob> {
    ensure!(!pattern.is_empty(), "empty glob");
    GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map_err(|error| anyhow!("contains invalid glob `{pattern}`: {error}"))
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum KeysOf {
    Format(KeyFormat),
    Options(KeyOptions),
}

impl KeysOf {
    fn options(&self) -> KeyOptions {
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
