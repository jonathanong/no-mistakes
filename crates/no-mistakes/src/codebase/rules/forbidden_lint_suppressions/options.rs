use super::RULE_ID;
use crate::codebase::rules::path_filter::GlobMatcher;
use anyhow::{ensure, Result};
use serde::Deserialize;

pub(super) const DEFAULT_DIRECTIVES: [&str; 6] = [
    "eslint-disable",
    "eslint-disable-line",
    "eslint-disable-next-line",
    "oxlint-disable",
    "oxlint-disable-line",
    "oxlint-disable-next-line",
];
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum Directive {
    #[serde(rename = "eslint-disable")]
    Eslint,
    #[serde(rename = "eslint-disable-line")]
    EslintLine,
    #[serde(rename = "eslint-disable-next-line")]
    EslintNext,
    #[serde(rename = "oxlint-disable")]
    Oxlint,
    #[serde(rename = "oxlint-disable-line")]
    OxlintLine,
    #[serde(rename = "oxlint-disable-next-line")]
    OxlintNext,
    #[serde(rename = "@ts-ignore")]
    TsIgnore,
    #[serde(rename = "@ts-expect-error")]
    TsExpectError,
}
impl Directive {
    pub(super) fn name(&self) -> &'static str {
        match self {
            Self::Eslint => "eslint-disable",
            Self::EslintLine => "eslint-disable-line",
            Self::EslintNext => "eslint-disable-next-line",
            Self::Oxlint => "oxlint-disable",
            Self::OxlintLine => "oxlint-disable-line",
            Self::OxlintNext => "oxlint-disable-next-line",
            Self::TsIgnore => "@ts-ignore",
            Self::TsExpectError => "@ts-expect-error",
        }
    }
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Options {
    pub files: Vec<String>,
    #[serde(default)]
    pub rule_ids: Vec<String>,
    #[serde(default)]
    pub rule_prefixes: Vec<String>,
    pub directives: Option<Vec<Directive>>,
    #[serde(default)]
    pub ban_blanket_disable: bool,
    pub message: Option<String>,
}
impl Options {
    pub(super) fn validate(&self) -> Result<GlobMatcher> {
        ensure!(
            !self.files.is_empty() && self.files.iter().all(|v| !v.trim().is_empty()),
            "{RULE_ID}: files must contain at least one non-empty glob"
        );
        ensure!(
            self.rule_ids
                .iter()
                .chain(&self.rule_prefixes)
                .all(|v| !v.trim().is_empty()),
            "{RULE_ID}: ruleIds and rulePrefixes must not contain empty entries"
        );
        GlobMatcher::new(&self.files, RULE_ID)
    }
    pub(super) fn enabled(&self, directive: &str) -> bool {
        self.directives.as_ref().map_or_else(
            || DEFAULT_DIRECTIVES.contains(&directive),
            |values| values.iter().any(|v| v.name() == directive),
        )
    }
    pub(super) fn banned(&self, rule: &str) -> bool {
        // A malformed comma item is not a blanket disable or a matching rule.
        !rule.chars().any(char::is_whitespace)
            && (self.rule_ids.iter().any(|v| v == rule)
                || self.rule_prefixes.iter().any(|v| rule.starts_with(v)))
    }
}
