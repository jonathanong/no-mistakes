use std::path::Path;

mod patterns;
use patterns::*;

use super::SwiftFileFacts;

pub(super) fn parse_swift_file_with_sources(
    path: &Path,
    target: Option<String>,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> Option<SwiftFileFacts> {
    let source = crate::codebase::ts_source::SourceStore::read_optional(sources, path)?;
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
            copy_string(&mut chars, &mut out);
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

fn copy_string(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>, out: &mut String) {
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
}

fn extract_imports(source: &str) -> Vec<String> {
    sorted_unique(
        swift_import_regex()
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    )
}

fn extract_declarations(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    out.extend(
        swift_declaration_regex()
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    );
    out.extend(
        swift_function_regex()
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    );
    out.extend(
        swift_property_regex()
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    );
    sorted_unique(out)
}

fn extract_references(source: &str) -> Vec<String> {
    let keywords = swift_reference_keywords();
    sorted_unique(
        swift_reference_regex()
            .captures_iter(source)
            .filter_map(|cap| {
                let raw = cap.get(0)?.as_str().trim_start_matches('.');
                (!keywords.contains(raw)).then(|| raw.to_string())
            }),
    )
}

fn extract_endpoint_paths(source: &str) -> Vec<String> {
    sorted_unique(
        swift_endpoint_path_regex()
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| swift_path_pattern(m.as_str()))),
    )
}

fn swift_path_pattern(path: &str) -> String {
    swift_interpolation_regex()
        .replace_all(path, "*")
        .into_owned()
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
