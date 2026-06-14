use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

use super::SwiftFileFacts;

pub(super) fn parse_swift_file(path: &Path, target: Option<String>) -> Option<SwiftFileFacts> {
    let source = std::fs::read_to_string(path).ok()?;
    let stripped = strip_comments(&source);
    Some(SwiftFileFacts {
        path: path.to_path_buf(),
        target,
        imports: extract_imports(&stripped),
        declarations: extract_declarations(&stripped),
        references: extract_references(&stripped),
        endpoint_paths: extract_endpoint_paths(&stripped),
    })
}

fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch == '"' {
            out.push(ch);
            copy_string(source, &mut chars, &mut out);
            continue;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '/') {
            chars.next();
            out.push(' ');
            out.push(' ');
            for (_, comment_ch) in chars.by_ref() {
                if comment_ch == '\n' {
                    out.push('\n');
                    break;
                }
                out.push(' ');
            }
            continue;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '*') {
            chars.next();
            out.push(' ');
            out.push(' ');
            let mut previous = '\0';
            for (_, comment_ch) in chars.by_ref() {
                if comment_ch == '\n' {
                    out.push('\n');
                } else {
                    out.push(' ');
                }
                if previous == '*' && comment_ch == '/' {
                    break;
                }
                previous = comment_ch;
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn copy_string(
    source: &str,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    out: &mut String,
) {
    let mut escaped = false;
    for (_, ch) in chars.by_ref() {
        out.push(ch);
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            break;
        }
    }
    if out.len() > source.len() {
        out.truncate(source.len());
    }
}

fn extract_imports(source: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)^\s*import\s+([A-Za-z_][A-Za-z0-9_]*)").expect("valid import regex");
    sorted_unique(
        re.captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    )
}

fn extract_declarations(source: &str) -> Vec<String> {
    let decl_re = Regex::new(r"\b(?:public\s+|internal\s+|private\s+|fileprivate\s+|open\s+|final\s+|static\s+|class\s+)*\b(?:struct|class|actor|enum|protocol|extension|typealias)\s+([A-Za-z_][A-Za-z0-9_]*)")
        .expect("valid declaration regex");
    let func_re = Regex::new(r"\b(?:static\s+|class\s+)?func\s+([A-Za-z_][A-Za-z0-9_]*)")
        .expect("valid function regex");
    let let_re = Regex::new(r"\b(?:static\s+|class\s+)?(?:let|var)\s+([A-Za-z_][A-Za-z0-9_]*)")
        .expect("valid property regex");
    let mut out = Vec::new();
    out.extend(
        decl_re
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    );
    out.extend(
        func_re
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    );
    out.extend(
        let_re
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    );
    sorted_unique(out)
}

fn extract_references(source: &str) -> Vec<String> {
    let ident_re = Regex::new(r"\b[A-Z_][A-Za-z0-9_]*\b|\.[A-Za-z_][A-Za-z0-9_]*\b")
        .expect("valid reference regex");
    let keywords: HashSet<&str> = [
        "Array",
        "Bool",
        "Data",
        "Dictionary",
        "Double",
        "Error",
        "False",
        "Float",
        "Int",
        "Nil",
        "Optional",
        "Result",
        "Self",
        "Set",
        "String",
        "True",
        "Void",
    ]
    .into_iter()
    .collect();
    sorted_unique(ident_re.captures_iter(source).filter_map(|cap| {
        let raw = cap.get(0)?.as_str().trim_start_matches('.');
        (!keywords.contains(raw)).then(|| raw.to_string())
    }))
}

fn extract_endpoint_paths(source: &str) -> Vec<String> {
    let re = Regex::new(r#"path\s*:\s*\"([^\"]+)\""#).expect("valid endpoint path regex");
    sorted_unique(
        re.captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| swift_path_pattern(m.as_str()))),
    )
}

fn swift_path_pattern(path: &str) -> String {
    let interpolation = Regex::new(r#"\\\([^)]*\)"#).expect("valid interpolation regex");
    interpolation.replace_all(path, "*").into_owned()
}

fn sorted_unique<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut out: Vec<String> = values.into_iter().collect();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests;
