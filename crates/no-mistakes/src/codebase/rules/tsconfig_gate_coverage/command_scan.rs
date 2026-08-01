//! Static command recognition shared by workflow and configured local gates.

/// Normalize one static repository-relative path into slash form.
/// Parent traversals, absolute paths, backslashes, and shell expansion syntax
/// are intentionally unresolved. A rule finding then asks the user to express
/// the project command statically instead of guessing its runtime value.
pub(crate) fn normalize_repo_relative(raw: &str) -> Option<String> {
    if raw.is_empty()
        || raw.starts_with('/')
        || raw.starts_with('~')
        || raw.contains('\\')
        || raw.contains(['$', '`', '(', ')'])
    {
        return None;
    }
    let mut parts = Vec::new();
    for part in raw.split('/') {
        match part {
            "" | "." => {}
            ".." => return None,
            part => parts.push(part),
        }
    }
    Some(if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    })
}

/// Scan a static workflow `run:` body for projects checked by `tsc --noEmit`.
///
/// Only sequential newline, `&&`, and `;` segments are recognized. Shell
/// interpolation, substitutions, pipes, conditionals, and arbitrary wrappers
/// are not evaluated. A reachability-affecting control command rejects the
/// whole body instead of trying to model shell execution.
pub(crate) fn scan_shell_for_typechecked_projects(script: &str, initial_cwd: &str) -> Vec<String> {
    let mut cwd = normalize_repo_relative(initial_cwd);
    let mut projects = Vec::new();
    for segment in script.split(['\n', ';']).flat_map(|line| line.split("&&")) {
        let Some(tokens) = static_tokens(segment) else {
            continue;
        };
        let Some(first) = tokens.first() else {
            continue;
        };
        if is_unsupported_control_command(first) {
            return Vec::new();
        }
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
        projects.extend(scan_tokens(&tokens, base));
    }
    projects.sort();
    projects.dedup();
    projects
}

/// These shell builtins can make later commands unreachable. Supporting their
/// control flow would require modeling shell semantics, so reject the entire
/// static body conservatively rather than crediting a possibly skipped `tsc`.
fn is_unsupported_control_command(command: &str) -> bool {
    matches!(command, "exit" | "return" | "false")
}

/// Scan one configured argv command. A shell script is accepted only for the
/// explicit `bash|sh -c <literal>` form; all other argv commands are parsed as
/// direct static command tokens.
pub(crate) fn scan_argv_for_typechecked_projects(argv: &[String], cwd: &str) -> Vec<String> {
    if argv.len() == 3 && matches!(argv[0].as_str(), "bash" | "sh") && argv[1] == "-c" {
        return scan_shell_for_typechecked_projects(&argv[2], cwd);
    }
    let Some(cwd) = normalize_repo_relative(cwd) else {
        return Vec::new();
    };
    scan_tokens(argv, &cwd).into_iter().collect()
}

fn scan_tokens(tokens: &[String], cwd: &str) -> Vec<String> {
    let Some((command, command_cwd)) = command_and_cwd(tokens, cwd) else {
        return Vec::new();
    };
    if !is_tsc(command)
        || !tokens.iter().any(|token| token == "--noEmit")
        || tokens.iter().any(|token| {
            matches!(
                token.as_str(),
                "--showConfig" | "--help" | "-h" | "--version" | "-v" | "--init"
            )
        })
    {
        return Vec::new();
    }
    let Some(project) = project_argument(tokens) else {
        return Vec::new();
    };
    join_relative(&command_cwd, &project).into_iter().collect()
}

/// `tsc` accepts one effective `--project` argument. Multiple or incomplete
/// spellings are intentionally unresolved rather than guessed.
fn project_argument(tokens: &[String]) -> Option<String> {
    let mut project = None;
    let mut index = 0;
    while let Some(token) = tokens.get(index) {
        let value = if let Some(value) = token.strip_prefix("--project=") {
            Some(value.to_string())
        } else if token == "-p" {
            return None;
        } else if token == "--project" {
            let value = tokens.get(index + 1)?.clone();
            index += 1;
            Some(value)
        } else {
            None
        };
        if let Some(value) = value {
            if project.replace(value).is_some() {
                return None;
            }
        }
        index += 1;
    }
    Some(project.unwrap_or_else(|| "tsconfig.json".to_string()))
}

fn command_and_cwd<'a>(tokens: &'a [String], cwd: &str) -> Option<(&'a str, String)> {
    match tokens.first()?.as_str() {
        "pnpm" => {
            let mut index = 1;
            let mut command_cwd = cwd.to_string();
            if let Some(value) = tokens
                .get(index)
                .and_then(|token| token.strip_prefix("--dir="))
            {
                command_cwd = join_relative(cwd, value)?;
                index += 1;
            } else if tokens.get(index).is_some_and(|token| token == "--dir") {
                command_cwd = join_relative(cwd, tokens.get(index + 1)?)?;
                index += 2;
            }
            (tokens.get(index)? == "exec").then_some((tokens.get(index + 1)?.as_str(), command_cwd))
        }
        command => Some((command, cwd.to_string())),
    }
}

fn is_tsc(command: &str) -> bool {
    command == "tsc" || command.ends_with("/tsc")
}

fn join_relative(base: &str, raw: &str) -> Option<String> {
    let raw = normalize_repo_relative(raw)?;
    let joined = if base == "." {
        raw
    } else {
        format!("{base}/{raw}")
    };
    normalize_repo_relative(&joined)
}

fn static_tokens(segment: &str) -> Option<Vec<String>> {
    if segment.is_empty()
        || segment.contains("||")
        || segment.contains('|')
        || segment.contains(['$', '`', '\\'])
    {
        return None;
    }
    let mut tokens = Vec::new();
    let mut chars = segment.chars().peekable();
    while chars.peek().is_some() {
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        let Some(first) = chars.next() else {
            break;
        };
        let mut token = String::new();
        if matches!(first, '\'' | '"') {
            let quote = first;
            let mut closed = false;
            for ch in chars.by_ref() {
                if ch == quote {
                    closed = true;
                    break;
                }
                token.push(ch);
            }
            if !closed || chars.peek().is_some_and(|ch| !ch.is_whitespace()) {
                return None;
            }
        } else {
            token.push(first);
            while chars.peek().is_some_and(|ch| !ch.is_whitespace()) {
                token.push(chars.next().expect("peeked character exists"));
            }
        }
        if token.is_empty() {
            return None;
        }
        tokens.push(token);
    }
    Some(tokens)
}

#[cfg(test)]
mod tests;
