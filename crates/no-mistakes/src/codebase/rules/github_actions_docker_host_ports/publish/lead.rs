use super::super::flags::{basename, is_assign};
use super::super::shell::Word;
use super::wrappers::{skip_adjusted, skip_env, skip_sudo, skip_xargs};

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
        if basename(text) == "sudo" {
            index = skip_sudo(words, index + 1);
            continue;
        }
        if basename(text) == "env" {
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
        let (flag, inline) = text
            .split_once('=')
            .map(|(name, value)| (name, Some(value)))
            .unwrap_or((text, None));
        if matches!(flag, "-D" | "--debug" | "--tls" | "--tlsverify") {
            index += 1;
            continue;
        }
        if !matches!(
            flag,
            "--context"
                | "-c"
                | "--host"
                | "-H"
                | "--config"
                | "--log-level"
                | "-l"
                | "--tlscacert"
                | "--tlscert"
                | "--tlskey"
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
