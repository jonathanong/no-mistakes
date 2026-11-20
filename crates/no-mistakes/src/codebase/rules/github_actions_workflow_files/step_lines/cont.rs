use super::cursor::Walker;
use super::flow;
use super::scalar;

pub(super) fn consume_block(walker: &mut Walker, raw: &str) -> bool {
    if raw.trim().is_empty() {
        walker.scalar.as_mut().unwrap().buf.push('\n');
        return true;
    }
    let indent = flow::leading(raw);
    let key_indent = walker.scalar.as_ref().unwrap().key_indent;
    let content = walker.scalar.as_ref().unwrap().content_indent;
    let inside = content.is_none_or(|content| indent >= content)
        && (content.is_some() || indent > key_indent);
    if !inside {
        scalar::finish_scalar(walker);
        return false;
    }
    let scalar = walker.scalar.as_mut().unwrap();
    if scalar.content_indent.is_none() {
        scalar.content_indent = Some(indent);
    }
    let cut = scalar.content_indent.unwrap_or(indent).min(raw.len());
    scalar.buf.push_str(&raw[cut..]);
    scalar.buf.push('\n');
    true
}

pub(super) fn consume_plain(walker: &mut Walker, raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        scalar::finish_scalar(walker);
        return false;
    }
    let indent = flow::leading(raw);
    let key_indent = walker.scalar.as_ref().unwrap().key_indent;
    if indent > key_indent {
        let scalar = walker.scalar.as_mut().unwrap();
        scalar.buf.push('\n');
        scalar.buf.push_str(trimmed);
        return true;
    }
    scalar::finish_scalar(walker);
    false
}

pub(super) fn consume_quote(walker: &mut Walker, raw: &str) -> bool {
    let trimmed = raw.trim();
    if !trimmed.is_empty() {
        let indent = flow::leading(raw);
        let key_indent = walker.scalar.as_ref().unwrap().key_indent;
        if indent <= key_indent {
            scalar::finish_scalar(walker);
            return false;
        }
    }
    let scalar = walker.scalar.as_mut().unwrap();
    scalar.buf.push('\n');
    scalar.buf.push_str(trimmed);
    if quote_closed(&scalar.buf) {
        scalar::finish_scalar(walker);
    }
    true
}

pub(super) fn quote_closed(text: &str) -> bool {
    let mut chars = text.chars().peekable();
    let Some(quote) = chars.next() else {
        return false;
    };
    if quote != '"' && quote != '\'' {
        return false;
    }
    while let Some(c) = chars.next() {
        if quote == '"' && c == '\\' {
            chars.next();
            continue;
        }
        if c != quote {
            continue;
        }
        if quote == '\'' && chars.peek() == Some(&'\'') {
            chars.next();
            continue;
        }
        let rest: String = chars.collect();
        let rest = rest.trim();
        return rest.is_empty() || rest.starts_with('#');
    }
    false
}

pub(super) fn unwrap_quotes(text: &str) -> String {
    let trimmed = text.trim();
    if !quote_closed(trimmed) {
        return trimmed.to_string();
    }
    let quote = trimmed.chars().next().unwrap_or('"');
    let mut chars = trimmed.chars().peekable();
    chars.next();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if quote == '"' && c == '\\' {
            if let Some(escaped) = chars.next() {
                out.push(escaped);
            }
            continue;
        }
        if c == quote {
            if quote == '\'' && chars.peek() == Some(&'\'') {
                chars.next();
                out.push('\'');
                continue;
            }
            break;
        }
        out.push(c);
    }
    out
}
