use super::{disable_directives, NamedDisableDirective};
use std::path::Path;

#[path = "rust_docs.rs"]
mod rust_docs;

/// Audit only comments in the host language, never documentation examples or literals.
pub(crate) fn audit_directives_for_path(
    source: &str,
    path: &Path,
) -> Vec<(NamedDisableDirective, bool)> {
    if !source.contains("no-mistakes-disable-") {
        return Vec::new();
    }
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if ["md", "markdown", "mdx"]
        .iter()
        .any(|ext| extension.eq_ignore_ascii_case(ext))
    {
        return ordinary(disable_directives(&markdown_comments(source), false));
    }
    if extension.eq_ignore_ascii_case("rs") {
        let (comments, blocks) = rust_comments(source);
        let mut found = ordinary(disable_directives(&comments, false));
        found.extend(
            rust_docs::directives(&comments, source, &blocks)
                .into_iter()
                .map(|item| (item, true)),
        );
        return found;
    }
    ordinary(disable_directives(source, true))
}

fn ordinary(items: Vec<NamedDisableDirective>) -> Vec<(NamedDisableDirective, bool)> {
    items.into_iter().map(|item| (item, false)).collect()
}

fn mask(bytes: &mut [u8]) {
    // The directive grammar treats a leading BOM as an invisible header prefix.
    let start = if bytes.starts_with(b"\xef\xbb\xbf") {
        3
    } else {
        0
    };
    for byte in &mut bytes[start..] {
        if !matches!(*byte, b'\n' | b'\r' | b' ' | b'\t') {
            *byte = b'x';
        }
    }
}

fn markdown_comments(source: &str) -> String {
    use pulldown_cmark::{Event, Options, Parser};
    let mut output = source.as_bytes().to_vec();
    mask(&mut output);
    // Offset events exclude fenced, indented, and inline code even inside lists
    // and blockquotes, while keeping the original physical directive locations.
    for (event, range) in Parser::new_ext(source, Options::all()).into_offset_iter() {
        if !matches!(event, Event::Html(_) | Event::InlineHtml(_)) {
            continue;
        }
        let html = &source[range.clone()];
        let mut cursor = 0;
        while let Some(start) = html[cursor..].find("<!--") {
            let start = cursor + start;
            let end = html[start..]
                .find("-->")
                .map_or(html.len(), |end| start + end + 3);
            output[range.start + start..range.start + end]
                .copy_from_slice(&html.as_bytes()[start..end]);
            cursor = end;
        }
    }
    String::from_utf8(output).expect("mask preserves UTF-8")
}

fn rust_comments(source: &str) -> (String, Vec<std::ops::Range<usize>>) {
    let mut docs = Vec::new();
    let bytes = source.as_bytes();
    let mut output = bytes.to_vec();
    mask(&mut output);
    let mut index = 0;
    let preamble = source.trim_start_matches('\u{FEFF}');
    if preamble.starts_with("#!") && !preamble.starts_with("#![") {
        let start = source.len() - preamble.len();
        output[start..start + 2].copy_from_slice(b"#!");
        index = start + preamble.find('\n').unwrap_or(preamble.len());
    }
    while index < bytes.len() {
        if bytes[index..].starts_with(b"//") {
            let start = index;
            index += bytes[index..]
                .iter()
                .position(|byte| *byte == b'\n')
                .unwrap_or(bytes.len() - index);
            output[start..index].copy_from_slice(&bytes[start..index]);
        } else if bytes[index..].starts_with(b"/*") {
            let start = index;
            let doc = bytes[index..].starts_with(b"/*!")
                || (bytes[index..].starts_with(b"/**")
                    && !bytes[index..].starts_with(b"/***")
                    && !bytes[index..].starts_with(b"/**/"));
            // Keep the outer framing so a license header is still a comment
            // before a file directive; nested framing stays masked.
            output[index..index + 2].copy_from_slice(b"/*");
            index += 2;
            let mut depth = 1;
            while index < bytes.len() && depth > 0 {
                if bytes[index..].starts_with(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    if depth == 0 {
                        output[index..index + 2].copy_from_slice(b"*/");
                    }
                    index += 2;
                } else {
                    index += 1;
                }
            }
            if doc {
                docs.push(start..index);
            }
        } else if let Some(end) = rust_literal_end(bytes, index) {
            index = end;
        } else {
            index += 1;
        }
    }
    (
        String::from_utf8(output).expect("mask preserves UTF-8"),
        docs,
    )
}

fn rust_literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes[start] == b'r' {
        let hashes = bytes[start + 1..]
            .iter()
            .take_while(|byte| **byte == b'#')
            .count();
        let quote = start + 1 + hashes;
        if bytes.get(quote) == Some(&b'"') {
            let mut end = quote + 1;
            while end < bytes.len() {
                if bytes[end] == b'"'
                    && bytes
                        .get(end + 1..end + 1 + hashes)
                        .is_some_and(|tail| tail.iter().all(|byte| *byte == b'#'))
                {
                    return Some(end + 1 + hashes);
                }
                end += 1;
            }
            return Some(bytes.len());
        }
    }
    let quote = bytes[start];
    if !matches!(quote, b'"' | b'\'') {
        return None;
    }
    // A Rust lifetime has no closing apostrophe after its identifier.
    if quote == b'\''
        && bytes
            .get(start + 1)
            .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_' || *byte >= 128)
    {
        let length = bytes[start + 1..]
            .iter()
            .take_while(|byte| byte.is_ascii_alphanumeric() || **byte == b'_' || **byte >= 128)
            .count();
        if bytes.get(start + 1 + length) != Some(&b'\'') {
            return None;
        }
    }
    let mut end = start + 1;
    while end < bytes.len() {
        if bytes[end] == b'\\' {
            end = (end + 2).min(bytes.len());
        } else if bytes[end] == quote {
            return Some(end + 1);
        } else {
            end += 1;
        }
    }
    Some(bytes.len())
}
