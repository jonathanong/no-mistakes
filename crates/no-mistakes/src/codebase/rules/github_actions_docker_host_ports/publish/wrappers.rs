use super::super::flags::{basename, is_assign};
use super::super::shell::Word;

pub(super) fn skip_sudo(words: &[Word], mut index: usize) -> usize {
    while let Some(flag) = words.get(index) {
        let text = flag.text.as_str();
        if text == "--" {
            return index + 1;
        }
        if matches!(
            text,
            "-E" | "-n" | "-H" | "-i" | "-s" | "-S" | "-k" | "-A" | "-b" | "-P" | "--preserve-env"
        ) || text.starts_with("--preserve-env=")
            || text.starts_with("--user=")
            || text.starts_with("--group=")
            || text.starts_with("--chdir=")
            || text.starts_with("--host=")
        {
            index += 1;
            continue;
        }
        if matches!(
            text,
            "-u" | "-g" | "-C" | "-h" | "--user" | "--group" | "--chdir" | "--host"
        ) {
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

pub(super) fn skip_env(words: &[Word], mut index: usize) -> usize {
    while let Some(word) = words.get(index) {
        let text = word.text.as_str();
        if matches!(text, "-C" | "--chdir" | "-S") {
            index += 1;
            if words.get(index).is_some() {
                index += 1;
            }
            continue;
        }
        if text.starts_with("--chdir=") {
            index += 1;
            continue;
        }
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
/// `-s SIG` and `-k DUR` consume their value before that duration.
pub(super) fn skip_adjusted(words: &[Word], mut index: usize) -> usize {
    while let Some(word) = words.get(index) {
        if !word.text.starts_with('-') {
            break;
        }
        let (flag, inline) = word
            .text
            .split_once('=')
            .map(|(name, value)| (name, Some(value)))
            .unwrap_or((word.text.as_str(), None));
        index += 1;
        if inline.is_none()
            && matches!(flag, "-s" | "-k" | "--signal" | "--kill-after")
            && words.get(index).is_some()
        {
            index += 1;
        }
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

pub(super) fn skip_xargs(words: &[Word], mut index: usize) -> usize {
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
        "-I" | "-n"
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
    )
}

/// `bash -c SCRIPT`, `sh -lc SCRIPT`, and `eval SCRIPT` hold a literal command.
pub(super) fn shell_operand(words: &[Word], mut index: usize) -> Option<&Word> {
    let name = basename(command_name(&words.get(index)?.text));
    if name == "eval" {
        let operand = words.get(index + 1)?;
        return (!operand.dynamic).then_some(operand);
    }
    if !matches!(name, "bash" | "sh" | "zsh") {
        return None;
    }
    index += 1;
    while let Some(flag) = words.get(index) {
        let text = flag.text.as_str();
        if let Some(operand) = c_operand(words, index, text) {
            return (!operand.dynamic).then_some(operand);
        }
        if !text.starts_with('-') {
            return None;
        }
        // `-O extglob` takes the next word, same as `-o` and `--rcfile`.
        index += if matches!(text, "-o" | "--rcfile" | "-O") {
            2
        } else {
            1
        };
    }
    None
}

fn c_operand<'a>(words: &'a [Word], index: usize, text: &str) -> Option<&'a Word> {
    if text == "-c" || text == "--command" {
        return words.get(index + 1);
    }
    let cluster = text
        .strip_prefix('-')
        .filter(|body| !body.starts_with('-'))?;
    let after = cluster.split_once('c')?.1;
    if after.is_empty() {
        words.get(index + 1)
    } else {
        None
    }
}

fn command_name(text: &str) -> &str {
    text.trim_start_matches('(').trim_end_matches(')')
}
