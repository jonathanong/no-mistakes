use super::lex::{assignment, command_words, Lexer, Shell};
use crate::codebase::command_lex::words::{split_quoted_words, Whitespace};
use std::path::Path;

pub const RULE_ID: &str = "zsh-readonly-parameter";

pub(super) fn likely_shell_file(path: &Path) -> bool {
    path.extension()
        .is_none_or(|ext| matches!(ext.to_str(), Some("sh" | "zsh")))
        || executable(path)
}

#[cfg(unix)]
fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|meta| meta.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn executable(_: &Path) -> bool {
    false
}

pub(super) fn has_shebang(source: &str) -> bool {
    let Some(header) = source
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("#!"))
    else {
        return false;
    };
    let Some(words) = split_quoted_words(header, Whitespace::Ascii) else {
        return false;
    };
    let mut words = words.iter().map(String::as_str);
    match words.next().and_then(basename) {
        Some("zsh") => true,
        Some("env") => env_interpreter(words).and_then(basename) == Some("zsh"),
        _ => false,
    }
}

fn env_interpreter<'a>(mut words: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    while let Some(word) = words.next() {
        let word = word
            .strip_prefix("--split-string=")
            .or_else(|| word.strip_prefix("-S"))
            .unwrap_or(word);
        match word {
            "" | "--split-string" | "-i" | "--ignore-environment" => {}
            "--" => return words.next(),
            _ => return Some(word),
        }
    }
    None
}

fn basename(interpreter: &str) -> Option<&str> {
    interpreter.rsplit('/').next()
}

pub(super) fn scan(source: &str) -> Vec<(&str, usize)> {
    let mut hits = Vec::new();
    let mut hidden_scope = None;
    Lexer::new(source, Shell::Zsh).commands(|words, scope| {
        if hidden_scope.is_some_and(|hidden| hidden > scope) {
            hidden_scope = None;
        }
        let mut command = command_words(words).peekable();
        while let Some(word) = command.next_if(|word| assignment(word.raw).is_some()) {
            if hidden_scope.is_none() && uses_status(word.raw) {
                hits.push(("status", word.line));
            }
        }
        if !command
            .next()
            .is_some_and(|word| matches!(word.raw, "local" | "typeset" | "declare"))
        {
            return;
        }
        let mut hide = false;
        let mut global = false;
        while let Some(word) = command.next_if(|word| word.raw.starts_with('-') && word.raw != "--")
        {
            let flags = &word.raw[1..];
            if flags.contains(['f', 'm', 'p']) {
                return;
            }
            hide |= flags.contains('h');
            global |= flags.contains('g');
        }
        command.next_if(|word| word.raw == "--");
        for word in command.filter(|word| uses_status(word.raw)) {
            if hide && !global && scope > 0 {
                hidden_scope.get_or_insert(scope);
            } else if hidden_scope.is_none() && (!hide || assignment(word.raw).is_some()) {
                hits.push(("status", word.line));
            }
        }
    });
    hits
}

fn uses_status(word: &str) -> bool {
    word == "status" || assignment(word).is_some_and(|(name, _)| name == "status")
}
