use anyhow::Result;
use regex::Regex;
use serde::Deserialize;

mod compile;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) commands: Vec<CommandPolicyOption>,
    pub(crate) include: Vec<String>,
    pub(crate) shell_files: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct CommandPolicyOption {
    pub(crate) command: Vec<String>,
    pub(crate) value_flags: Vec<String>,
    pub(crate) when_any_args: Vec<ArgOption>,
    pub(crate) unless_any_args: Vec<ArgOption>,
    pub(crate) require_args: Vec<ArgOption>,
    pub(crate) require_any_args: Vec<ArgOption>,
    pub(crate) forbid_arg_patterns: Vec<ForbiddenPatternOption>,
    pub(crate) message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum ArgOption {
    Single(String),
    Sequence(Vec<String>),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct ForbiddenPatternOption {
    pub(crate) pattern: String,
    #[serde(default)]
    pub(crate) scope: ArgScope,
    #[serde(default)]
    pub(crate) message: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ArgScope {
    #[default]
    Any,
    Positional,
}

pub(in crate::codebase::rules) struct CompiledOptions {
    pub(super) commands: Vec<CompiledPolicy>,
    pub(super) include: Vec<String>,
    pub(super) shell_files: Vec<String>,
}

pub(crate) struct CompiledPolicy {
    pub(crate) command: Vec<String>,
    pub(crate) label: String,
    pub(crate) value_flags: Vec<String>,
    pub(crate) when_any_args: Vec<ArgMatcher>,
    pub(crate) unless_any_args: Vec<ArgMatcher>,
    pub(crate) require_args: Vec<ArgMatcher>,
    pub(crate) require_any_args: Vec<ArgMatcher>,
    pub(crate) forbid_arg_patterns: Vec<CompiledForbiddenPattern>,
    pub(crate) message: Option<String>,
}

pub(crate) struct ArgMatcher {
    pub(crate) tokens: Vec<String>,
    pub(crate) label: String,
}

pub(crate) struct CompiledForbiddenPattern {
    pub(crate) raw: String,
    pub(crate) regex: Regex,
    pub(crate) scope: ArgScope,
    pub(crate) message: Option<String>,
}

pub(in crate::codebase::rules) fn compile(options: Options) -> Result<CompiledOptions> {
    compile::compile(options)
}
