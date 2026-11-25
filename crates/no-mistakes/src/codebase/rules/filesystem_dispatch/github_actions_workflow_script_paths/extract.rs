use crate::codebase::rules::github_actions_download_integrity::words::{self, Word};

const SHELL_VAL_FLAGS: &[&str] = &["-o", "-O", "--rcfile"];
const SHELL_INLINE_FLAGS: &[&str] = &["-c", "--command"];
const NODE_VAL_FLAGS: &[&str] = &[
    "-r",
    "--require",
    "--import",
    "--loader",
    "--env-file",
    "--env-file-if-exists",
    "--conditions",
    "-C",
    "--test-reporter",
    "--title",
];
const NODE_INLINE_FLAGS: &[&str] = &["-e", "--eval", "-p", "--print", "-c", "--check"];
const TSX_VAL_FLAGS: &[&str] = &[
    "--tsconfig",
    "-P",
    "--project",
    "--env-file",
    "-r",
    "--require",
];
const TSX_INLINE_FLAGS: &[&str] = &["-e", "--eval", "-p", "--print"];
const SUBCMD_VAL_FLAGS: &[&str] = &[
    "--cwd",
    "--config",
    "-c",
    "--env-file",
    "--allow-read",
    "--allow-write",
];

pub(super) fn shell_c_operand(words: &[Word], mut index: usize) -> Option<&Word> {
    while let Some(word) = words.get(index) {
        let text = word.text.as_str();
        if text == "-c" || text == "--command" {
            return words.get(index + 1);
        }
        if let Some(cluster) = text.strip_prefix('-').filter(|body| !body.starts_with('-')) {
            if let Some((_, after)) = cluster.split_once('c') {
                return after.is_empty().then(|| words.get(index + 1)).flatten();
            }
        }
        if !text.starts_with('-') {
            return None;
        }
        index += 1 + usize::from(matches!(text, "-o" | "-O" | "--rcfile"));
    }
    None
}

pub(super) fn interpreter_script_arg<'a>(
    name: &str,
    words: &'a [Word],
    start: usize,
) -> Option<&'a Word> {
    match name {
        "bash" | "sh" | "zsh" | "dash" | "ash" | "ksh" => {
            first_script_arg(words, start, SHELL_VAL_FLAGS, SHELL_INLINE_FLAGS)
        }
        "source" | "." => words.get(start),
        "node" => first_script_arg(words, start, NODE_VAL_FLAGS, NODE_INLINE_FLAGS),
        "tsx" | "ts-node" => first_script_arg(words, start, TSX_VAL_FLAGS, TSX_INLINE_FLAGS),
        "python" | "python3" => first_script_arg(
            words,
            start,
            &["-W", "-X", "--check-hash-based-pycs"],
            &["-c", "-m"],
        ),
        "ruby" | "perl" => first_script_arg(words, start, &["-I", "-r"], &["-e", "-E"]),
        "pwsh" => first_script_arg(
            words,
            start,
            &["-WorkingDirectory", "-ConfigurationName"],
            &["-c", "-Command"],
        )
        .or_else(|| flag_value(words, start, &["-f", "-File"])),
        "bun" | "deno" => unwrap_subcommand_runner(name, words, start),
        "npx" | "pnpx" | "pnpm" | "npm" | "yarn" | "uv" => {
            unwrap_package_runner(name, words, start)
        }
        _ => None,
    }
}

fn unwrap_subcommand_runner<'a>(name: &str, words: &'a [Word], start: usize) -> Option<&'a Word> {
    let mut idx = start;
    while let Some(word) = words.get(idx) {
        let text = word.text.as_str();
        if text == "--" {
            return words.get(idx + 1);
        }
        if matches!(text, "-e" | "--eval") {
            return None;
        }
        if text.starts_with('-') {
            idx += 1 + usize::from(matches!(text, "--cwd" | "--config" | "-c" | "--env-file"));
            continue;
        }
        if text == "run" || (name == "bun" && text == "x") {
            return first_script_arg(words, idx + 1, SUBCMD_VAL_FLAGS, &["-e", "--eval"]);
        }
        return Some(word);
    }
    None
}

fn unwrap_package_runner<'a>(name: &str, words: &'a [Word], start: usize) -> Option<&'a Word> {
    let mut idx = start;
    while let Some(word) = words.get(idx) {
        let text = word.text.as_str();
        if text == "--" {
            idx += 1;
            break;
        }
        if text.starts_with('-') {
            idx += 1 + usize::from(matches!(
                text,
                "-C" | "--dir" | "--prefix" | "-F" | "--filter" | "--package" | "-p"
            ));
            continue;
        }
        if (matches!(name, "pnpm" | "npm" | "yarn") && matches!(text, "exec" | "dlx"))
            || (name == "uv" && text == "run")
        {
            idx += 1;
            continue;
        }
        break;
    }
    let next = words.get(idx)?;
    if next.text.contains('/') && !next.text.contains("://") {
        return Some(next);
    }
    let nested = words::basename(&next.text);
    (nested != name)
        .then(|| interpreter_script_arg(&nested, words, idx + 1))
        .flatten()
}

fn first_script_arg<'a>(
    words: &'a [Word],
    mut index: usize,
    value_flags: &[&str],
    inline_flags: &[&str],
) -> Option<&'a Word> {
    while let Some(word) = words.get(index) {
        let text = word.text.as_str();
        if text == "--" {
            return words.get(index + 1);
        }
        if inline_flags.contains(&text) {
            return None;
        }
        if value_flags.contains(&text) || shell_short_value_flag(text, value_flags) {
            index += 2;
            continue;
        }
        if text.starts_with('-') {
            index += 1;
            continue;
        }
        return Some(word);
    }
    None
}

fn flag_value<'a>(words: &'a [Word], index: usize, flags: &[&str]) -> Option<&'a Word> {
    let pos = words[index..].iter().position(|word| {
        flags
            .iter()
            .any(|flag| word.text.eq_ignore_ascii_case(flag))
    })?;
    words.get(index + pos + 1)
}

fn shell_short_value_flag(text: &str, value_flags: &[&str]) -> bool {
    let Some(cluster) = text.strip_prefix('-').filter(|body| !body.starts_with('-')) else {
        return false;
    };
    cluster.len() > 1
        && value_flags.iter().any(|flag| {
            flag.len() == 2
                && flag.starts_with('-')
                && cluster.ends_with(flag.as_bytes()[1] as char)
        })
}
