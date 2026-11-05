use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckFailure {
    pub consumer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum Prerequisite<T> {
    Ready(T),
    Failed(Vec<CheckFailure>),
}

impl CheckFailure {
    pub fn new(consumer: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            consumer: consumer.into(),
            file: None,
            property: None,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CheckInputFailure {
    pub file: Option<PathBuf>,
    pub property: Option<String>,
    pub message: String,
}

impl CheckInputFailure {
    pub fn source(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            file: Some(file.into()),
            property: None,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CheckInputFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CheckInputFailure {}

#[doc(hidden)]
pub fn check_application_key(rule: &crate::config::v2::schema::RuleDef, ordinal: usize) -> String {
    format!(
        "codebase.application.{}.{}",
        rule.rule,
        rule.resolved_application_index.unwrap_or(ordinal)
    )
}
