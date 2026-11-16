use super::{directive_payload, parse_payload, BoundComment};

#[path = "bound_comments_template_host.rs"]
mod host;

/// `--` inside a template literal is SQL. A backtick inside a host string,
/// comment, or regex does not open that template.
pub(super) fn template_sql_comments(source: &str, rule_id: &str) -> Vec<BoundComment> {
    let mut found = Vec::new();
    let mut carry = Carry {
        regex_ok: true,
        ..Carry::default()
    };
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
    quote: Option<char>,
    escaped: bool,
    regex: bool,
    regex_class: bool,
    regex_ok: bool,
    /// Brace depth of each JS template opened inside an interpolation.
    /// Zero is template contents, where a quote is not a host string.
    js: Vec<u32>,
}

fn directive<'a>(line: &'a str, carry: &mut Carry) -> Option<&'a str> {
    // A trailing `\` consumed the newline, so the next character stays literal.
    if carry.escaped {
        carry.escaped = false;
    }
    // A host regex cannot contain a raw newline. Drop it so it cannot swallow the file.
    if carry.regex {
        carry.regex = false;
        carry.regex_class = false;
    }
    let mut ident = String::new();
    let mut found = None;
    let mut chars = line.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if carry.quote.is_some() {
            host::quote_char(ch, carry);
            continue;
        }
        if carry.block {
            host::close_block(ch, &mut chars, carry);
            continue;
        }
        if carry.regex {
            host::regex_char(ch, carry);
            continue;
        }
        if carry.js.last() == Some(&0) {
            js_content(ch, &mut chars, carry);
            continue;
        }
        if carry.template && carry.braces == 0 {
            if template_step(ch, &mut chars, carry) {
                continue;
            }
            if found.is_none() && ch == '-' && super::super::dash_comment_can_start(line, idx) {
                found = Some(line[idx + 2..].trim());
            }
            continue;
        }
        if !host::host_step(line, idx, ch, &mut chars, carry, &mut ident) {
            break;
        }
    }
    host::finish_ident(&mut ident, carry);
    found
}

fn js_content(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    carry: &mut Carry,
) {
    if carry.escaped {
        carry.escaped = false;
        return;
    }
    if ch == '\\' {
        carry.escaped = true;
        return;
    }
    if ch == '`' {
        carry.js.pop();
        return;
    }
    if ch == '$' && chars.peek().is_some_and(|(_, next)| *next == '{') {
        chars.next();
        // `js_content` runs only while a frame is open.
        *carry.js.last_mut().expect("nested template") = 1;
        carry.regex_ok = true;
    }
}

fn template_step(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    carry: &mut Carry,
) -> bool {
    if carry.escaped {
        carry.escaped = false;
        return true;
    }
    if ch == '\\' {
        carry.escaped = true;
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
        carry.regex_ok = true;
        return true;
    }
    false
}
