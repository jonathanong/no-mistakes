use serde_yaml::Value;

mod runner_labels;
pub(super) use runner_labels::{
    container_runner_support, has_static_runnable_runs_on, runner_os,
    runs_on_can_default_to_windows, runs_on_has_statically_invalid_value, ContainerRunnerSupport,
};

#[cfg(test)]
mod tests;

fn default_shell(value: &Value) -> Option<&str> {
    value
        .get("defaults")
        .and_then(|defaults| defaults.get("run"))
        .and_then(|run| run.get("shell"))
        .and_then(Value::as_str)
}

/// Returns the most-specific static shell setting. `None` means GitHub
/// Actions' implicit shell, which preserves the rule's existing behavior.
pub(super) fn effective_shell(value: &Value, fallback: Option<String>) -> Option<String> {
    match value.get("shell").and_then(Value::as_str) {
        Some(shell) => Some(shell.to_string()),
        None => default_shell(value).map(str::to_string).or(fallback),
    }
}

/// Return whether a supported shell preserves failures for every command in a
/// multi-command body. Built-in and implicit Actions shells provide `-e`; a
/// custom template must express `-e` or `-o errexit` itself.
pub(super) fn shell_failure_enforced(shell: Option<&str>) -> Option<bool> {
    let Some(shell) = shell else {
        return Some(true);
    };
    let mut tokens = shell.split_ascii_whitespace();
    let command = tokens.next()?;
    if !matches!(command, "bash" | "sh") {
        return None;
    }
    let args = tokens.collect::<Vec<_>>();
    args.is_empty()
        .then_some(true)
        .or_else(|| execution_preserving_shell_template_failure_enforced(command, &args))
}

pub(super) fn shell_pipefail_enforced(shell: Option<&str>) -> bool {
    let Some(shell) = shell else {
        return false;
    };
    let mut tokens = shell.split_ascii_whitespace();
    if tokens.next() != Some("bash") {
        return false;
    }
    let arguments = tokens.collect::<Vec<_>>();
    arguments.is_empty()
        || arguments
            .windows(2)
            .any(|pair| is_bash_pipefail_option(pair[0]) && pair.get(1) == Some(&"pipefail"))
}

fn execution_preserving_shell_template_failure_enforced(
    command: &str,
    arguments: &[&str],
) -> Option<bool> {
    if arguments.last() != Some(&"{0}") {
        return None;
    }
    let options = &arguments[..arguments.len() - 1];
    let mut index = 0;
    let mut failure_enforced = false;
    while let Some(option) = options.get(index) {
        match *option {
            "--noprofile" | "--norc" if command == "bash" => index += 1,
            option
                if command == "bash"
                    && is_bash_pipefail_option(option)
                    && options.get(index + 1) == Some(&"pipefail") =>
            {
                failure_enforced |= option.contains('e');
                index += 2;
            }
            "-o" if options.get(index + 1) == Some(&"errexit") => {
                failure_enforced = true;
                index += 2;
            }
            option if let Some(enforced) = execution_preserving_short_option(option) => {
                failure_enforced |= enforced;
                index += 1;
            }
            _ => return None,
        }
    }
    Some(failure_enforced)
}

fn is_bash_pipefail_option(option: &str) -> bool {
    let Some(flags) = option.strip_prefix('-') else {
        return false;
    };
    let Some(prefix) = flags.strip_suffix('o') else {
        return false;
    };
    prefix.chars().all(|flag| matches!(flag, 'e' | 'u' | 'x'))
}

/// `-e`, `-u`, and `-x` only affect error handling or diagnostics. `-o` is
/// handled separately so it can be limited to Bash's execution-safe pipefail.
fn execution_preserving_short_option(option: &str) -> Option<bool> {
    let flags = option.strip_prefix('-')?;
    (!flags.is_empty() && flags.chars().all(|flag| matches!(flag, 'e' | 'u' | 'x')))
        .then_some(flags.contains('e'))
}
