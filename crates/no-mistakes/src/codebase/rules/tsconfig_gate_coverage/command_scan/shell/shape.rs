use super::super::{normalize_repo_relative, static_tokens};
use super::comments::strip_static_comments;

mod effective_command;
mod first_word;
pub(super) use effective_command::effective_tokens;
use effective_command::{effective_first_word, has_leading_redirection};
use first_word::{has_dangling_escape, has_dynamic_first_word};

pub(crate) fn shell_body_has_safe_static_shape(script: &str) -> bool {
    let script = strip_static_comments(script);
    if script.contains("||") || contains_unsupported_multiline_shell_construct(&script) {
        return false;
    }
    for group in script
        .split(['\n', ';'])
        .filter(|group| !group.trim().is_empty())
    {
        for segment in group
            .split("&&")
            .filter(|segment| !segment.trim().is_empty())
        {
            if !pipeline_has_safe_static_shape(segment) {
                return false;
            }
        }
    }
    true
}

fn pipeline_has_safe_static_shape(segment: &str) -> bool {
    segment.split('|').all(|command| {
        if has_dangling_escape(command) || has_dynamic_first_word(command) {
            return false;
        }
        let Some(tokens) = static_tokens(command) else {
            return !unparseable_segment_is_unsafe(command);
        };
        !is_unsupported_shape_control(&tokens) && !is_unsupported_working_directory_command(&tokens)
    })
}

fn is_unsupported_shape_control(tokens: &[String]) -> bool {
    has_leading_redirection(tokens)
        || (!matches!(
            tokens.first().map(String::as_str),
            Some("false" | "exit" | "return")
        ) && is_unsupported_control_command(tokens))
}

pub(super) fn is_unsupported_control_command(tokens: &[String]) -> bool {
    if has_leading_redirection(tokens) {
        return true;
    }
    let Some(command) = effective_tokens(tokens).and_then(|tokens| tokens.first()) else {
        return false;
    };
    is_shell_affecting_command(command)
}

fn is_shell_affecting_command(command: &str) -> bool {
    matches!(
        command,
        "!" | "."
            | "alias"
            | "case"
            | "coproc"
            | "do"
            | "done"
            | "elif"
            | "else"
            | "esac"
            | "declare"
            | "enable"
            | "eval"
            | "exec"
            | "exit"
            | "false"
            | "fi"
            | "export"
            | "for"
            | "function"
            | "hash"
            | "if"
            | "getopts"
            | "kill"
            | "let"
            | "local"
            | "logout"
            | "mapfile"
            | "printf"
            | "read"
            | "readarray"
            | "readonly"
            | "return"
            | "select"
            | "set"
            | "shift"
            | "shopt"
            | "source"
            | "suspend"
            | "then"
            | "time"
            | "trap"
            | "typeset"
            | "ulimit"
            | "umask"
            | "unalias"
            | "unset"
            | "until"
            | "while"
    )
}

fn unparseable_segment_is_unsafe(command: &str) -> bool {
    let Some(command) = effective_first_word(command) else {
        return true;
    };
    let command = command.replace('\\', "");
    is_shell_affecting_command(&command)
        || matches!(
            command.as_str(),
            "alias"
                | "builtin"
                | "cd"
                | "command"
                | "declare"
                | "dirs"
                | "enable"
                | "export"
                | "hash"
                | "getopts"
                | "kill"
                | "let"
                | "local"
                | "logout"
                | "mapfile"
                | "popd"
                | "printf"
                | "pushd"
                | "read"
                | "readarray"
                | "readonly"
                | "set"
                | "shift"
                | "shopt"
                | "trap"
                | "typeset"
                | "ulimit"
                | "umask"
                | "unalias"
                | "unset"
        )
}

fn is_unsupported_working_directory_command(tokens: &[String]) -> bool {
    let Some(tokens) = effective_tokens(tokens) else {
        return true;
    };
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
    let mut escaped = false;
    let mut characters = script.chars().peekable();
    while let Some(character) = characters.next() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        match quote {
            Some(active) if character == active => quote = None,
            Some(_) if matches!(character, '\n' | ';' | '&') => return true,
            Some(_) => {}
            None if matches!(character, '\'' | '"') => quote = Some(character),
            None if matches!(character, '{' | '}' | '(' | ')' | '<' | '>') => return true,
            None if character == '&' && characters.next_if_eq(&'&').is_none() => return true,
            None => {}
        }
    }
    quote.is_some()
}
