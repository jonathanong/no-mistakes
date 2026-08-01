use serde_yaml::Value;

#[cfg(test)]
mod tests;

/// A CI job cannot provide a typecheck gate unless Actions can schedule it on
/// a statically known runner. Reusable-workflow jobs use `uses:` rather than
/// `steps:` and are excluded separately by the step requirement.
pub(super) fn has_static_runnable_runs_on(job: &Value) -> bool {
    match job.get("runs-on") {
        Some(Value::String(label)) => is_static_runner_label(label),
        Some(Value::Sequence(labels)) => {
            !labels.is_empty()
                && labels
                    .iter()
                    .all(|label| label.as_str().is_some_and(is_static_runner_label))
        }
        _ => false,
    }
}

fn is_static_runner_label(label: &str) -> bool {
    !label.trim().is_empty() && !label.contains("${{")
}

/// An unspecified Actions shell is PowerShell on Windows. Only reject this
/// known incompatible default; an explicit supported `bash`/`sh` override is
/// still safe to analyze on the same runner.
pub(super) fn runs_on_can_default_to_windows(job: &Value) -> bool {
    match job.get("runs-on") {
        Some(Value::String(label)) => is_windows_runner_label(label),
        Some(Value::Sequence(labels)) => labels
            .iter()
            .filter_map(Value::as_str)
            .any(is_windows_runner_label),
        _ => false,
    }
}

fn is_windows_runner_label(label: &str) -> bool {
    let label = label.trim();
    label.eq_ignore_ascii_case("windows")
        || label
            .get(.."windows-".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("windows-"))
}

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
