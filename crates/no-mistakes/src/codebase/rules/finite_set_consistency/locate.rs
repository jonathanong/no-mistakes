//! Best-effort 1-based source line of the key or declaration a set was read
//! from, so findings point at the compared YAML/JSON key or TS declaration
//! instead of line 1. An unresolved key falls back to the nearest resolved
//! ancestor, then line 1; it is never matched at the wrong depth.

use super::comments::{strip_comments, strip_sql_comments};
use super::SetSpec;
use regex::Regex;

pub(super) fn set_line(spec: &SetSpec, source: &str) -> usize {
    match spec.kind.as_str() {
        "yaml-sequence" => {
            let path = spec.key.split('.').map(String::from).collect::<Vec<_>>();
            yaml_key_line(source, &path)
        }
        "yaml-string-selector" => {
            super::yaml::selector_key_path(&spec.key).map_or(1, |path| yaml_key_line(source, &path))
        }
        "ts-string-union"
        | "ts-const-object-keys"
        | "ts-const-object-property"
        | "ts-array-literal"
        | "ts-const-array-property" => declaration_line(
            &strip_comments(source),
            r"\b(?:type|const|let|var|enum|interface|function|class)\s+",
            &spec.target,
        ),
        "sql-enum" => declaration_line(
            &strip_sql_comments(source),
            r"(?i)\bCREATE\s+TYPE\s+",
            &spec.target,
        ),
        _ => 1,
    }
}

/// `source` has comments removed with newlines kept, so offsets map to lines.
fn declaration_line(source: &str, prefix: &str, target: &str) -> usize {
    if target.is_empty() {
        return 1;
    }
    let pattern = format!(r"{prefix}{}\b", regex::escape(target));
    Regex::new(&pattern)
        .ok()
        .and_then(|re| re.find(source))
        .map_or(1, |found| source[..found.start()].matches('\n').count() + 1)
}

/// A content line outside comments and block scalar bodies.
struct Entry<'a> {
    line: usize,
    indent: usize,
    /// Column where the mapping key starts, after any `- ` markers.
    col: usize,
    dash: bool,
    text: &'a str,
}

fn entries(source: &str) -> Vec<Entry<'_>> {
    let mut out: Vec<Entry<'_>> = Vec::new();
    let mut block: Option<usize> = None;
    for (index, raw) in source.lines().enumerate() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let indent = raw.len() - trimmed.len();
        if block.is_some_and(|col| indent > col) {
            continue;
        }
        block = None;
        if trimmed.starts_with('#') {
            continue;
        }
        let mut col = indent;
        let mut text = trimmed;
        let dash = text == "-" || text.starts_with("- ");
        while let Some(next) = text.strip_prefix("- ") {
            let rest = next.trim_start();
            col += text.len() - rest.len();
            text = rest;
        }
        if text == "-" {
            text = "";
        }
        if opens_block_scalar(text) {
            block = Some(col);
        }
        out.push(Entry {
            line: index + 1,
            indent,
            col,
            dash,
            text,
        });
    }
    out
}

fn opens_block_scalar(text: &str) -> bool {
    let value = text.rsplit_once(": ").map_or(text, |(_, value)| value);
    let mut chars = value.trim_end().chars();
    matches!(chars.next(), Some('|' | '>'))
        && chars.all(|c| c == '+' || c == '-' || c.is_ascii_digit())
}

fn key_matches(text: &str, key: &str) -> bool {
    [
        format!("{key}:"),
        format!("\"{key}\":"),
        format!("'{key}':"),
    ]
    .iter()
    .any(|prefix| {
        text.strip_prefix(prefix.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
    })
}

/// Entries owned by the previously matched segment.
#[derive(Clone, Copy)]
struct Scope<'a, 'b> {
    entries: &'b [Entry<'a>],
    /// The first entry is a `- ` item line whose inline key is a direct child.
    item: bool,
}

fn yaml_key_line(source: &str, path: &[String]) -> usize {
    let all = entries(source);
    // A JSON document opens with a bare brace; its keys are the next level.
    let start = usize::from(all.first().is_some_and(|first| first.text == "{"));
    let mut scope = Scope {
        entries: &all[start..],
        item: false,
    };
    let mut found = 1;
    for segment in path {
        let hit = match segment.parse::<usize>() {
            Ok(nth) => nth_item(scope, nth),
            Err(_) => find_key(scope, segment),
        };
        let Some((line, next)) = hit else {
            break;
        };
        found = line;
        scope = next;
    }
    found
}

/// Direct child mapping key `key` of `scope`, with the value scope it owns.
fn find_key<'a, 'b>(scope: Scope<'a, 'b>, key: &str) -> Option<(usize, Scope<'a, 'b>)> {
    let region = scope.entries;
    let first = region.first()?;
    let level = match (scope.item, first.text.is_empty()) {
        (true, false) => first.col,
        (true, true) => region.get(1)?.indent,
        (false, _) if first.dash => return None,
        (false, _) => first.indent,
    };
    let at = region.iter().enumerate().position(|(index, entry)| {
        let direct = if scope.item && index == 0 {
            entry.col == level
        } else {
            entry.indent == level && !entry.dash
        };
        direct && key_matches(entry.text, key)
    })?;
    let owner = &region[at];
    let end = region[at + 1..]
        .iter()
        .position(|entry| !(entry.indent > owner.col || (entry.indent == owner.col && entry.dash)))
        .map_or(region.len(), |offset| at + 1 + offset);
    let next = Scope {
        entries: &region[at + 1..end],
        item: false,
    };
    Some((owner.line, next))
}

/// The `nth` direct `- ` item of `scope`, including its own line.
fn nth_item<'a, 'b>(scope: Scope<'a, 'b>, nth: usize) -> Option<(usize, Scope<'a, 'b>)> {
    let region = scope.entries;
    let level = region.first().filter(|entry| entry.dash)?.indent;
    let at = region
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.dash && entry.indent == level)
        .map(|(index, _)| index)
        .nth(nth)?;
    let end = region[at + 1..]
        .iter()
        .position(|entry| entry.indent <= level)
        .map_or(region.len(), |offset| at + 1 + offset);
    let next = Scope {
        entries: &region[at..end],
        item: true,
    };
    Some((region[at].line, next))
}
