use std::ops::Range;

#[derive(Clone, Copy, Eq, PartialEq)]
enum SyntaxKind {
    Code,
    String,
    Comment,
}

pub fn mask_comments(source: &str) -> String {
    mask_syntax(source, false)
}

pub fn mask_comments_and_strings(source: &str) -> String {
    mask_syntax(source, true)
}

pub fn find_outside_syntax(source: &str, needle: &str, offset: usize) -> Option<usize> {
    if needle.is_empty() || offset >= source.len() {
        return None;
    }

    let mut found = None;
    scan_syntax(source, |kind, range| {
        if found.is_some() || kind != SyntaxKind::Code {
            return;
        }

        let start = range.start.max(offset);
        if start >= range.end {
            return;
        }

        if let Some(relative) = source[start..range.end].find(needle) {
            found = Some(start + relative);
        }
    });
    found
}

fn mask_syntax(source: &str, include_strings: bool) -> String {
    let mut masked = source.as_bytes().to_vec();
    scan_syntax(source, |kind, range| {
        if kind == SyntaxKind::Comment || (include_strings && kind == SyntaxKind::String) {
            mask_range(&mut masked, range);
        }
    });
    String::from_utf8(masked).expect("syntax masking preserves UTF-8")
}

fn mask_range(masked: &mut [u8], range: Range<usize>) {
    for i in range {
        if !matches!(masked[i], b'\n' | b'\r') {
            masked[i] = b' ';
        }
    }
}

fn scan_syntax(source: &str, mut visit: impl FnMut(SyntaxKind, Range<usize>)) {
    let bytes = source.as_bytes();
    let mut code_start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b'\'' | b'"' | b'`' => {
                if code_start < i {
                    visit(SyntaxKind::Code, code_start..i);
                }
                let end = string_end(bytes, i);
                visit(SyntaxKind::String, i..end);
                i = end;
                code_start = i;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                if code_start < i {
                    visit(SyntaxKind::Code, code_start..i);
                }
                let end = line_comment_end(bytes, i + 2);
                visit(SyntaxKind::Comment, i..end);
                i = end;
                code_start = i;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                if code_start < i {
                    visit(SyntaxKind::Code, code_start..i);
                }
                let end = block_comment_end(bytes, i + 2);
                visit(SyntaxKind::Comment, i..end);
                i = end;
                code_start = i;
            }
            _ => i += 1,
        }
    }

    if code_start < bytes.len() {
        visit(SyntaxKind::Code, code_start..bytes.len());
    }
}

fn string_end(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut escaped = false;
    let mut i = start + 1;

    while i < bytes.len() {
        if escaped {
            escaped = false;
        } else if bytes[i] == b'\\' {
            escaped = true;
        } else if bytes[i] == quote {
            return i + 1;
        }
        i += 1;
    }

    i
}

fn line_comment_end(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && !matches!(bytes[i], b'\n' | b'\r') {
        i += 1;
    }
    i
}

fn block_comment_end(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() {
        if bytes[i] == b'*' && bytes.get(i + 1) == Some(&b'/') {
            return i + 2;
        }
        i += 1;
    }
    i
}
