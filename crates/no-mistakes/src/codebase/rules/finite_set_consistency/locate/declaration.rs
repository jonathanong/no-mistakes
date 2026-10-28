use regex::Regex;

/// `source` has comments removed with newlines kept, so offsets map to lines.
pub(super) fn declaration_line(source: &str, prefix: &str, target: &str) -> usize {
    if target.is_empty() {
        return 1;
    }
    let pattern = format!(r"{prefix}{}\b", regex::escape(target));
    Regex::new(&pattern)
        .ok()
        .and_then(|re| re.find(source))
        .map_or(1, |found| source[..found.start()].matches('\n').count() + 1)
}

/// Replaces string and template literal contents with spaces, keeping the
/// quotes and every newline so line numbers survive. A `'` or `"` string ends
/// at an unescaped newline, so a stray apostrophe cannot blank the rest of the
/// file.
pub(super) fn blank_strings(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for ch in source.chars() {
        let Some(open) = quote else {
            if matches!(ch, '"' | '\'' | '`') {
                quote = Some(ch);
            }
            out.push(ch);
            continue;
        };
        if ch == '\n' {
            out.push('\n');
            if !escaped && open != '`' {
                quote = None;
            }
            escaped = false;
        } else if escaped {
            escaped = false;
            out.push(' ');
        } else if ch == '\\' {
            escaped = true;
            out.push(' ');
        } else if ch == open {
            quote = None;
            out.push(ch);
        } else {
            out.push(' ');
        }
    }
    out
}
