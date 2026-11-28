use super::super::RULE_ID;
use super::{
    ArgMatcher, ArgOption, CommandPolicyOption, CompiledForbiddenPattern, CompiledOptions,
    CompiledPolicy, Options,
};
use crate::codebase::rules::path_filter::GlobMatcher;
use anyhow::{bail, ensure, Context, Result};
use regex::Regex;

pub(super) fn compile(options: Options) -> Result<CompiledOptions> {
    validate_globs(&options.include, "include")?;
    validate_globs(&options.shell_files, "shellFiles")?;
    let mut commands = Vec::with_capacity(options.commands.len());
    for (index, entry) in options.commands.into_iter().enumerate() {
        commands.push(compile_policy(entry, index)?);
    }
    Ok(CompiledOptions {
        commands,
        include: options.include,
        shell_files: options.shell_files,
    })
}

fn validate_globs(globs: &[String], field: &str) -> Result<()> {
    for (pos, glob) in globs.iter().enumerate() {
        ensure!(
            !glob.trim().is_empty(),
            "{RULE_ID}: options.{field}[{pos}] must be a non-empty glob pattern"
        );
    }
    GlobMatcher::new(globs, &format!("{RULE_ID} options.{field}"))?;
    Ok(())
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
    let unless_any_args = compile_args(
        entry.unless_any_args,
        index,
        "unlessAnyArgs",
        &mut value_flags,
    )?;
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
        unless_any_args,
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
