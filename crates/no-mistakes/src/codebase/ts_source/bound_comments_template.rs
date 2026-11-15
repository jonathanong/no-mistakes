use super::{directive_payload, parse_payload, BoundComment};

/// `--` inside a template literal is SQL. A backtick inside a host string
/// or comment does not open that template.
pub(super) fn template_sql_comments(source: &str, rule_id: &str) -> Vec<BoundComment> {
    let mut found = Vec::new();
    let mut carry = Carry::default();
    for (index, line) in source.trim_start_matches('\u{FEFF}').lines().enumerate() {
        let Some(body) = directive(line, &mut carry) else {
            continue;
        };
        let Some(payload) = directive_payload(body, rule_id) else {
            continue;
        };
        found.push(parse_payload((index + 1) as u32, payload));
    }
    found
}

#[derive(Default)]
struct Carry {
    template: bool,
    braces: u32,
    sql_string: bool,
    block: bool,
}

fn directive<'a>(line: &'a str, carry: &mut Carry) -> Option<&'a str> {
    let mut template_escaped = false;
    let mut host_quote = None;
    let mut host_escaped = false;
    let mut found = None;
    let mut chars = line.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if carry.braces > 0 {
            adjust_braces(ch, carry);
            continue;
        }
        if carry.block {
            close_block(ch, &mut chars, carry);
            continue;
        }
        if !carry.template {
            if host_step(
                line,
                idx,
                ch,
                &mut chars,
                &mut host_quote,
                &mut host_escaped,
                carry,
            ) {
                continue;
            }
            break;
        }
        if template_step(ch, &mut chars, &mut template_escaped, carry) {
            continue;
        }
        if found.is_none() && ch == '-' && super::super::dash_comment_can_start(line, idx) {
            found = Some(line[idx + 2..].trim());
        }
    }
    found
}

fn adjust_braces(ch: char, carry: &mut Carry) {
    if ch == '{' {
        carry.braces += 1;
    } else if ch == '}' {
        carry.braces -= 1;
        if carry.braces == 0 {
            carry.template = true;
        }
    }
}

fn close_block(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    carry: &mut Carry,
) {
    if ch == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
        chars.next();
        carry.block = false;
    }
}

fn host_step(
    line: &str,
    idx: usize,
    ch: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    host_quote: &mut Option<char>,
    host_escaped: &mut bool,
    carry: &mut Carry,
) -> bool {
    if let Some(quote) = *host_quote {
        if *host_escaped {
            *host_escaped = false;
        } else if ch == '\\' {
            *host_escaped = true;
        } else if ch == quote {
            *host_quote = None;
        }
        return true;
    }
    let next = chars.peek().map(|(_, next)| *next);
    if ch == '/' && next == Some('*') {
        chars.next();
        carry.block = true;
        return true;
    }
    if ch == '/' && next == Some('/') {
        return false;
    }
    if ch == '#' && super::super::comment_prefix_can_start(line, idx) {
        return false;
    }
    if ch == '-' && next == Some('-') && super::super::dash_comment_can_start(line, idx) {
        return false;
    }
    if ch == '\'' || ch == '"' {
        *host_quote = Some(ch);
        return true;
    }
    if ch == '`' {
        carry.template = true;
    }
    true
}

fn template_step(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    template_escaped: &mut bool,
    carry: &mut Carry,
) -> bool {
    if *template_escaped {
        *template_escaped = false;
        return true;
    }
    if ch == '\\' {
        *template_escaped = true;
        return true;
    }
    if carry.sql_string {
        if ch == '\'' {
            if chars.peek().is_some_and(|(_, next)| *next == '\'') {
                chars.next();
            } else {
                carry.sql_string = false;
            }
        }
        return true;
    }
    if ch == '\'' {
        carry.sql_string = true;
        return true;
    }
    if ch == '`' {
        carry.template = false;
        carry.sql_string = false;
        return true;
    }
    if ch == '$' && chars.peek().is_some_and(|(_, next)| *next == '{') {
        chars.next();
        carry.braces = 1;
        carry.template = false;
        return true;
    }
    false
}
