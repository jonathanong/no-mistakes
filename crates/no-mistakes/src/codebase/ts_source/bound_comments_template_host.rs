use super::Carry;

pub(super) fn quote_char(ch: char, carry: &mut Carry) {
    if carry.escaped {
        carry.escaped = false;
    } else if ch == '\\' {
        carry.escaped = true;
    } else if Some(ch) == carry.quote {
        carry.quote = None;
        carry.regex_ok = false;
    }
}

pub(super) fn close_block(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    carry: &mut Carry,
) {
    if ch == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
        chars.next();
        carry.block = false;
    }
}

pub(super) fn regex_char(ch: char, carry: &mut Carry) {
    if carry.escaped {
        carry.escaped = false;
        return;
    }
    if ch == '\\' {
        carry.escaped = true;
        return;
    }
    if carry.regex_class {
        if ch == ']' {
            carry.regex_class = false;
        }
        return;
    }
    if ch == '[' {
        carry.regex_class = true;
        return;
    }
    if ch == '/' {
        carry.regex = false;
        carry.regex_ok = false;
    }
}

pub(super) fn host_step(
    line: &str,
    idx: usize,
    ch: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    carry: &mut Carry,
    ident: &mut String,
) -> bool {
    if !is_ident(ch) {
        finish_ident(ident, carry);
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
    if ch == '/' && carry.regex_ok {
        carry.regex = true;
        carry.regex_class = false;
        carry.escaped = false;
        return true;
    }
    if ch == '#' && super::super::super::comment_prefix_can_start(line, idx) {
        return false;
    }
    if ch == '-' && next == Some('-') && super::super::super::dash_comment_can_start(line, idx) {
        return false;
    }
    if ch == '\'' || ch == '"' {
        carry.quote = Some(ch);
        carry.escaped = false;
        carry.regex_ok = false;
        return true;
    }
    if ch == '`' && carry.braces == 0 {
        carry.template = true;
        carry.regex_ok = false;
        return true;
    }
    if is_ident(ch) {
        ident.push(ch);
        carry.regex_ok = false;
        return true;
    }
    if carry.braces > 0 && (ch == '{' || ch == '}') {
        adjust_braces(ch, carry);
        carry.regex_ok = ch == '{';
        return true;
    }
    if !ch.is_whitespace() {
        // Operators that may be followed by a regex literal. `-` is not a range.
        carry.regex_ok = "([{,;:?=!&|+-*%<>~^/".contains(ch);
    }
    true
}

fn is_ident(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

pub(super) fn finish_ident(ident: &mut String, carry: &mut Carry) {
    if ident.is_empty() {
        return;
    }
    carry.regex_ok = matches!(
        ident.as_str(),
        "await"
            | "case"
            | "delete"
            | "do"
            | "else"
            | "in"
            | "instanceof"
            | "new"
            | "of"
            | "return"
            | "throw"
            | "typeof"
            | "void"
            | "yield"
    );
    ident.clear();
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
