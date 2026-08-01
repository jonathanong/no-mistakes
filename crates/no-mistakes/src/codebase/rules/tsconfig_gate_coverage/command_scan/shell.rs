use super::{join_relative, normalize_repo_relative, scan_tokens, static_tokens};

pub(super) fn scan_shell_body_for_typechecked_projects(
    script: &str,
    initial_cwd: &str,
    mut failure_enforced: bool,
) -> Vec<String> {
    if contains_unsupported_multiline_shell_construct(script) {
        return Vec::new();
    }
    let mut cwd = normalize_repo_relative(initial_cwd);
    let mut projects = Vec::new();
    let segments = script
        .split(['\n', ';'])
        .flat_map(|line| line.split("&&"))
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>();
    for (index, segment) in segments.iter().enumerate() {
        let Some(tokens) = static_tokens(segment) else {
            continue;
        };
        let first = tokens
            .first()
            .expect("a nonblank static shell segment has at least one token");
        if is_unsupported_control_command(first)
            || disables_failure_enforcement(&tokens)
            || is_unsupported_working_directory_command(&tokens)
        {
            return Vec::new();
        }
        failure_enforced |= enables_failure_enforcement(&tokens);
        if first == "cd" {
            cwd = (tokens.len() == 2)
                .then(|| {
                    tokens
                        .get(1)
                        .and_then(|path| cwd.as_ref().and_then(|base| join_relative(base, path)))
                })
                .flatten();
            continue;
        }
        let Some(base) = cwd.as_deref() else {
            continue;
        };
        if failure_enforced || index + 1 == segments.len() {
            projects.extend(scan_tokens(&tokens, base));
        }
    }
    projects.sort();
    projects.dedup();
    projects
}

/// The scanner tracks only `cd <static-relative-path>`. Directory-stack
/// commands and malformed `cd` forms make a later command's cwd ambiguous, so
/// reject the whole body instead of crediting it against the wrong tsconfig.
fn is_unsupported_working_directory_command(tokens: &[String]) -> bool {
    match tokens.first().map(String::as_str) {
        Some("pushd" | "popd" | "dirs") => true,
        Some("cd") => {
            tokens.len() != 2
                || normalize_repo_relative(tokens.get(1).expect("cd has an argument")).is_none()
        }
        _ => false,
    }
}

fn contains_unsupported_multiline_shell_construct(script: &str) -> bool {
    if script.contains("<<") {
        return true;
    }
    let mut quote = None;
    for character in script.chars() {
        match quote {
            Some(active) if character == active => quote = None,
            Some(_) if character == '\n' => return true,
            Some(_) => {}
            None if matches!(character, '\'' | '"') => quote = Some(character),
            None => {}
        }
    }
    false
}

fn is_unsupported_control_command(command: &str) -> bool {
    matches!(command, "exit" | "return" | "false")
}

fn disables_failure_enforcement(tokens: &[String]) -> bool {
    if tokens.first().is_none_or(|command| command != "set") {
        return false;
    }
    match tokens.get(1).map(String::as_str) {
        Some(option) if option.starts_with('+') && option.contains('e') => true,
        Some("+o") => tokens.get(2).is_some_and(|option| option == "errexit"),
        _ => false,
    }
}

fn enables_failure_enforcement(tokens: &[String]) -> bool {
    if tokens.first().is_none_or(|command| command != "set") {
        return false;
    }
    match tokens.get(1).map(String::as_str) {
        Some(option) if option.starts_with('-') && option.contains('e') => true,
        Some("-o") => tokens.get(2).is_some_and(|option| option == "errexit"),
        _ => false,
    }
}

/// Parse the explicit local `bash|sh ... -c <literal>` shape. Unlike Actions,
/// local shells start without failure propagation unless `-e`/`errexit` is
/// present; the scanner can still credit a final `tsc` command.
pub(super) fn local_shell_command(argv: &[String]) -> Option<(&str, bool)> {
    if !matches!(argv.first()?.as_str(), "bash" | "sh") {
        return None;
    }
    let mut failure_enforced = false;
    let mut index = 1;
    while let Some(argument) = argv.get(index) {
        if argument == "-c" {
            if index + 2 != argv.len() {
                return None;
            }
            return Some((argv.get(index + 1)?.as_str(), failure_enforced));
        }
        if argument == "-o" || argument == "+o" {
            let option = argv.get(index + 1)?;
            if option != "errexit" {
                return None;
            }
            failure_enforced = argument == "-o";
            index += 2;
            continue;
        }
        let (prefix, options) = argument
            .strip_prefix('-')
            .map(|options| ("-", options))
            .or_else(|| argument.strip_prefix('+').map(|options| ("+", options)))?;
        if options.is_empty() || !options.chars().all(|option| option.is_ascii_alphabetic()) {
            return None;
        }
        if options.contains('c') {
            if prefix != "-" || options.matches('c').count() != 1 || index + 2 != argv.len() {
                return None;
            }
            failure_enforced |= options.contains('e');
            return Some((argv.get(index + 1)?.as_str(), failure_enforced));
        }
        if options.contains('e') {
            failure_enforced = prefix == "-";
        }
        index += 1;
    }
    None
}
