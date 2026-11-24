use crate::codebase::rules::github_actions_workflow_files::shell_words;

pub(crate) use shell_words::Word;

pub(super) fn vitest_argv(script: &str) -> Vec<Vec<Word>> {
    shell_words::commands(script)
        .commands
        .into_iter()
        .filter_map(|command| invocation(&command))
        .collect()
}

fn invocation(command: &[Word]) -> Option<Vec<Word>> {
    let mut index = 0;
    while index < command.len() {
        let text = &command[index].text;
        let base = basename(text);
        if is_assignment(text) || matches!(base, "(" | "{" | ")" | "}") || is_shell_keyword(base) {
            index += 1;
            continue;
        }
        if matches!(base, "sudo" | "command" | "time" | "nice" | "nohup" | "env") {
            index += 1;
            index = skip_flags(command, index, base == "env");
            continue;
        }
        break;
    }
    let launcher = basename(&command.get(index)?.text);
    if launcher == "vitest" {
        return Some(command[index + 1..].to_vec());
    }
    if matches!(launcher, "pnpm" | "npm" | "npx" | "yarn" | "bun" | "bunx") {
        return launched(command, index, launcher);
    }
    None
}

fn launched(command: &[Word], start: usize, launcher: &str) -> Option<Vec<Word>> {
    let mut index = start + 1;
    let mut exec = false;
    while index < command.len() {
        let text = &command[index].text;
        if text == "--" {
            index += 1;
            exec = true;
            continue;
        }
        if text.starts_with('-') {
            let takes = flag_takes_value(launcher, text) && !text.contains('=');
            index += 1 + usize::from(takes);
            continue;
        }
        if !exec && matches!(text.as_str(), "exec" | "x" | "dlx") {
            index += 1;
            exec = true;
            continue;
        }
        if !exec && text == "run" {
            return (basename(&command.get(index + 1)?.text) == "vitest")
                .then(|| command[index + 2..].to_vec());
        }
        if basename(text) == "vitest" {
            return Some(command[index + 1..].to_vec());
        }
        if exec || matches!(launcher, "npx" | "bunx") {
            index += 1;
            continue;
        }
        return None;
    }
    None
}

fn is_shell_keyword(base: &str) -> bool {
    matches!(
        base,
        "if" | "elif" | "then" | "else" | "do" | "while" | "until"
    )
}

fn skip_flags(command: &[Word], mut index: usize, assignments: bool) -> usize {
    while index < command.len() {
        let text = &command[index].text;
        if text.starts_with('-') {
            index += 1;
            continue;
        }
        if assignments && is_assignment(text) {
            index += 1;
            continue;
        }
        break;
    }
    index
}

fn flag_takes_value(launcher: &str, token: &str) -> bool {
    let flag = token.split('=').next().unwrap_or(token);
    matches!(
        (launcher, flag),
        ("pnpm", "--filter" | "-F" | "--dir" | "-C" | "--workspace")
            | (
                "npm" | "npx",
                "--prefix" | "--userconfig" | "--cache" | "--package" | "-p"
            )
            | ("yarn", "--cwd")
            | ("bun" | "bunx", "--filter" | "--cwd" | "-F")
    )
}

fn basename(text: &str) -> &str {
    text.rsplit(['/', '\\']).next().unwrap_or(text)
}

pub(super) fn is_assignment(text: &str) -> bool {
    let Some((name, _)) = text.split_once('=') else {
        return false;
    };
    let mut chars = name.chars();
    matches!(chars.next(), Some(ch) if ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
