use super::super::super::yaml_parse::parse_line;
use super::super::super::yaml_text::leading_indent;

pub(super) fn count_lines(header: usize, body: &[(usize, String)], source_lines: &[&str]) -> usize {
    let raw = source_lines[header - 1];
    let value = raw.split_once(':').map_or("", |(_, value)| value.trim());
    let quote = value.chars().next().filter(|ch| matches!(ch, '\'' | '"'));
    let block_indent = block_indent(raw, value, body);
    let mut count = 0;
    let mut continued = false;
    for (index, (line, text)) in body.iter().enumerate() {
        if block_indent.is_some_and(|indent| {
            text.trim_start().starts_with('#') && leading_indent(text) < indent
        }) {
            continue;
        }
        let payload = quote.map_or(text.as_str(), |quote| {
            quoted_payload(text, quote, index == 0)
        });
        if !payload.trim().is_empty() && !continued {
            count += 1;
        }
        continued = source_lines
            .get(line - 1)
            .is_some_and(|text| text.ends_with('\\'));
    }
    count
}

fn block_indent(raw: &str, value: &str, body: &[(usize, String)]) -> Option<usize> {
    if !value.starts_with(['|', '>']) {
        return None;
    }
    let key_indent = parse_line(raw).map_or(leading_indent(raw), |parsed| parsed.key_indent);
    let indicator = value
        .split_whitespace()
        .next()
        .unwrap_or("")
        .bytes()
        .find(|byte| byte.is_ascii_digit());
    if let Some(indicator) = indicator {
        return Some(key_indent + usize::from(indicator - b'0'));
    }
    body.iter()
        .find(|(_, text)| !text.trim().is_empty())
        .map(|(_, text)| leading_indent(text))
}

fn quoted_payload(text: &str, quote: char, first: bool) -> &str {
    let text = if first {
        text.trim_start().strip_prefix(quote).unwrap_or(text)
    } else {
        text
    };
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if quote == '"' && ch == '\\' {
            chars.next();
        } else if ch == quote {
            if quote == '\'' && chars.peek().is_some_and(|(_, next)| *next == '\'') {
                chars.next();
            } else {
                return &text[..index];
            }
        }
    }
    text
}
