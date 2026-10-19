use super::super::flags::is_assign;
use super::super::shell::Word;

const LEADING: &[&str] = &[
    "if", "then", "elif", "else", "do", "while", "until", "!", "{", "(", "time", "exec", "command",
    "nohup",
];

pub(super) fn command_text(text: &str) -> &str {
    text.trim_start_matches('(').trim_end_matches(')')
}

pub(super) fn skip_prefix(words: &[Word]) -> usize {
    let mut index = 0;
    while let Some(word) = words.get(index) {
        let text = command_text(&word.text);
        if text.is_empty() || is_assign(&word.text) || is_assign(text) || LEADING.contains(&text) {
            index += 1;
            continue;
        }
        if text == "sudo" {
            index = skip_sudo(words, index + 1);
            continue;
        }
        if text == "env" {
            index = skip_env(words, index + 1);
            continue;
        }
        if matches!(text, "timeout" | "nice" | "stdbuf") {
            index = skip_adjusted(words, index + 1);
            continue;
        }
        if text == "xargs" {
            index = skip_xargs(words, index + 1);
            continue;
        }
        break;
    }
    index
}

/// Docker client globals sit between `docker` and `run` / `create`.
pub(super) fn skip_globals(words: &[Word], mut index: usize) -> usize {
    while let Some(word) = words.get(index) {
        let text = word.text.as_str();
        if text == "-D" || text == "--debug" {
            index += 1;
            continue;
        }
        let (flag, inline) = text
            .split_once('=')
            .map(|(name, value)| (name, Some(value)))
            .unwrap_or((text, None));
        if !matches!(
            flag,
            "--context" | "--host" | "--config" | "--log-level" | "-H"
        ) {
            break;
        }
        index += 1;
        if inline.is_none() && words.get(index).is_some() {
            index += 1;
        }
    }
    index
}

fn skip_sudo(words: &[Word], mut index: usize) -> usize {
    while let Some(flag) = words.get(index) {
        let text = flag.text.as_str();
        if matches!(text, "-E" | "-n" | "-H" | "--preserve-env")
            || text.starts_with("--preserve-env=")
        {
            index += 1;
            continue;
        }
        if text == "-u" {
            index += 1;
            if words.get(index).is_some() {
                index += 1;
            }
            continue;
        }
        break;
    }
    index
}

fn skip_env(words: &[Word], mut index: usize) -> usize {
    while let Some(word) = words.get(index) {
        let text = word.text.as_str();
        if is_assign(text) || matches!(text, "-i" | "--ignore-environment") {
            index += 1;
            continue;
        }
        if matches!(text, "-u" | "--unset") {
            index += 1;
            if words.get(index).is_some() {
                index += 1;
            }
            continue;
        }
        if text.starts_with("--unset=") {
            index += 1;
            continue;
        }
        break;
    }
    index
}

/// `timeout` / `nice` take one numeric argument. `stdbuf` is flags only.
fn skip_adjusted(words: &[Word], mut index: usize) -> usize {
    while words
        .get(index)
        .is_some_and(|word| word.text.starts_with('-'))
    {
        index += 1;
    }
    if words
        .get(index)
        .is_some_and(|word| is_wrapper_value(&word.text))
    {
        index += 1;
    }
    index
}

fn is_wrapper_value(text: &str) -> bool {
    let body = text.strip_prefix('+').unwrap_or(text);
    let body = body.trim_end_matches(['s', 'm', 'h', 'd']);
    !body.is_empty() && body.bytes().all(|byte| byte.is_ascii_digit())
}

fn skip_xargs(words: &[Word], mut index: usize) -> usize {
    while let Some(word) = words.get(index) {
        let text = word.text.as_str();
        if !text.starts_with('-') {
            break;
        }
        index += 1;
        if xargs_flag_takes_value(text)
            && words
                .get(index)
                .is_some_and(|next| !next.text.starts_with('-'))
        {
            index += 1;
        }
    }
    index
}

fn xargs_flag_takes_value(text: &str) -> bool {
    if text.contains('=') {
        return false;
    }
    matches!(
        text,
        "-I" | "-i"
            | "-n"
            | "-P"
            | "-s"
            | "-a"
            | "-d"
            | "-E"
            | "-L"
            | "-J"
            | "--max-args"
            | "--max-procs"
            | "--max-chars"
            | "--delimiter"
            | "--arg-file"
            | "--replace"
            | "--eof"
    )
}
