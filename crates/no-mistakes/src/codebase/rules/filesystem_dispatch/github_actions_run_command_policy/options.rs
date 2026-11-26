use super::RULE_ID;
use anyhow::{bail, ensure, Context, Result};
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) commands: Vec<CommandPolicyOption>,
    pub(crate) include: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct CommandPolicyOption {
    pub(crate) command: Vec<String>,
    pub(crate) value_flags: Vec<String>,
    pub(crate) when_any_args: Vec<ArgOption>,
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
}

pub(super) struct CompiledPolicy {
    pub(super) command: Vec<String>,
    pub(super) label: String,
    pub(super) value_flags: Vec<String>,
    pub(super) when_any_args: Vec<ArgMatcher>,
    pub(super) require_args: Vec<ArgMatcher>,
    pub(super) require_any_args: Vec<ArgMatcher>,
    pub(super) forbid_arg_patterns: Vec<CompiledForbiddenPattern>,
    pub(super) message: Option<String>,
}

pub(super) struct ArgMatcher {
    pub(super) tokens: Vec<String>,
    pub(super) label: String,
}

pub(super) struct CompiledForbiddenPattern {
    pub(super) raw: String,
    pub(super) regex: Regex,
    pub(super) scope: ArgScope,
    pub(super) message: Option<String>,
}

pub(in crate::codebase::rules) fn compile(options: Options) -> Result<CompiledOptions> {
    crate::codebase::rules::file_matching::compile_globs(
        options.include.iter().map(String::as_str),
    )
    .context("options.include")?;
    let mut commands = Vec::with_capacity(options.commands.len());
    for (index, entry) in options.commands.into_iter().enumerate() {
        commands.push(compile_policy(entry, index)?);
    }
    Ok(CompiledOptions {
        commands,
        include: options.include,
    })
}

fn compile_policy(entry: CommandPolicyOption, index: usize) -> Result<CompiledPolicy> {
    ensure!(
        !entry.command.is_empty(),
        "{RULE_ID}: options.commands[{index}].command must be a non-empty list of command tokens"
    );
    for (pos, token) in entry.command.iter().enumerate() {
        ensure!(
            !token.is_empty() && !token.chars().any(char::is_whitespace),
            "{RULE_ID}: options.commands[{index}].command[{pos}] must be a non-empty token without whitespace"
        );
    }
    let mut value_flags = Vec::new();
    for (pos, flag) in entry.value_flags.into_iter().enumerate() {
        ensure!(
            flag.starts_with('-')
                && flag.len() > 1
                && flag != "--"
                && !flag.contains('=')
                && !flag.chars().any(char::is_whitespace),
            "{RULE_ID}: options.commands[{index}].valueFlags[{pos}] `{flag}` must be a flag name like `-f` or `--method`"
        );
        if !value_flags.contains(&flag) {
            value_flags.push(flag);
        }
    }
    let when_any_args = compile_args(entry.when_any_args, index, "whenAnyArgs", &mut value_flags)?;
    let require_args = compile_args(entry.require_args, index, "requireArgs", &mut value_flags)?;
    let require_any_args = compile_args(
        entry.require_any_args,
        index,
        "requireAnyArgs",
        &mut value_flags,
    )?;
    let mut forbid_arg_patterns = Vec::with_capacity(entry.forbid_arg_patterns.len());
    for (pos, item) in entry.forbid_arg_patterns.into_iter().enumerate() {
        ensure!(
            !item.pattern.is_empty(),
            "{RULE_ID}: options.commands[{index}].forbidArgPatterns[{pos}].pattern must be non-empty"
        );
        let regex = Regex::new(&item.pattern).with_context(|| {
            format!(
                "{RULE_ID}: invalid regex in options.commands[{index}].forbidArgPatterns[{pos}].pattern"
            )
        })?;
        forbid_arg_patterns.push(CompiledForbiddenPattern {
            raw: item.pattern,
            regex,
            scope: item.scope,
            message: normalize_message(item.message),
        });
    }
    if require_args.is_empty() && require_any_args.is_empty() && forbid_arg_patterns.is_empty() {
        bail!(
            "{RULE_ID}: options.commands[{index}] must configure at least one of requireArgs, requireAnyArgs, or forbidArgPatterns"
        );
    }
    let label = entry.command.join(" ");
    Ok(CompiledPolicy {
        command: entry.command,
        label,
        value_flags,
        when_any_args,
        require_args,
        require_any_args,
        forbid_arg_patterns,
        message: normalize_message(entry.message),
    })
}

fn compile_args(
    raw: Vec<ArgOption>,
    policy_index: usize,
    field: &str,
    value_flags: &mut Vec<String>,
) -> Result<Vec<ArgMatcher>> {
    let mut out = Vec::with_capacity(raw.len());
    for (pos, arg) in raw.into_iter().enumerate() {
        let tokens = match arg {
            ArgOption::Single(token) => vec![token],
            ArgOption::Sequence(tokens) => tokens,
        };
        ensure!(
            !tokens.is_empty() && tokens.iter().all(|token| !token.is_empty()),
            "{RULE_ID}: options.commands[{policy_index}].{field}[{pos}] must contain non-empty argument strings"
        );
        if tokens.len() == 2 {
            let head = &tokens[0];
            if head.starts_with('-')
                && head.len() > 1
                && head != "--"
                && !head.contains('=')
                && !value_flags.contains(head)
            {
                value_flags.push(head.clone());
            }
        }
        let label = tokens.join(" ");
        out.push(ArgMatcher { tokens, label });
    }
    Ok(out)
}

fn normalize_message(message: Option<String>) -> Option<String> {
    message
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}
