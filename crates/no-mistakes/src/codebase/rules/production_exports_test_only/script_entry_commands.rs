use crate::codebase::shell_words::{split_quoted_words, Whitespace};
use std::path::Path;

pub(super) fn command_segments(script: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut chars = script.chars().peekable();
    while let Some(character) = chars.next() {
        if let Some(open) = quote {
            current.push(character);
            if character == open {
                quote = None;
            }
            continue;
        }
        if character == '\'' || character == '"' {
            quote = Some(character);
            current.push(character);
            continue;
        }
        let paired = matches!(character, '&' | '|') && chars.peek() == Some(&character);
        if paired || character == ';' || character == '\n' {
            if paired {
                chars.next();
            }
            push_segment(&mut segments, &mut current);
            continue;
        }
        current.push(character);
    }
    push_segment(&mut segments, &mut current);
    segments
}

fn push_segment(segments: &mut Vec<String>, current: &mut String) {
    if current.trim().is_empty() {
        current.clear();
    } else {
        segments.push(std::mem::take(current));
    }
}

const VALUE_FLAGS: &[&str] = &[
    "-o",
    "--outfile",
    "--outdir",
    "--out-dir",
    "--out",
    "--config",
    "-c",
    "--format",
    "--platform",
    "--target",
    "--tsconfig",
];

pub(super) fn positional_entries(segment: &str) -> Vec<String> {
    let Some(words) = split_quoted_words(segment, Whitespace::Unicode) else {
        return Vec::new();
    };
    let mut bundler = false;
    let mut skip_value = false;
    let mut entries = Vec::new();
    for word in words {
        if skip_value {
            skip_value = false;
            continue;
        }
        if is_environment_assignment(&word) || word.starts_with('-') {
            if !word.contains('=') && VALUE_FLAGS.contains(&word.as_str()) {
                skip_value = true;
            }
            continue;
        }
        if is_bundler(&word) {
            bundler = true;
            continue;
        }
        if bundler && is_code_path(&word) {
            entries.push(word);
        }
    }
    entries
}

fn is_bundler(word: &str) -> bool {
    matches!(
        command_base(word),
        "esbuild" | "tsup" | "vite" | "rollup" | "webpack" | "wrangler"
    )
}

fn command_base(word: &str) -> &str {
    Path::new(word)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(word)
}

fn is_code_path(word: &str) -> bool {
    Path::new(word)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| crate::codebase::ts_source::TS_JS_EXTENSIONS.contains(&extension))
}

fn is_environment_assignment(token: &str) -> bool {
    let Some((name, _)) = token.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && name
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
}
