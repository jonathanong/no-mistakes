use super::Carry;

#[path = "bound_comments_template_host_value.rs"]
mod value;

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
    // Postfix `--` stays code when a space follows. A host `--` comment still ends the line.
    if ch == '-' && next == Some('-') && host_postfix_decrement(line, idx, carry) {
        chars.next();
        carry.regex_ok = false;
        return true;
    }
    if ch == '-' && next == Some('-') && super::super::super::dash_comment_can_start(line, idx) {
        return false;
    }
    value::value_step(ch, next, chars, carry, ident)
}

fn host_postfix_decrement(line: &str, idx: usize, carry: &Carry) -> bool {
    let prev = line[..idx].trim_end().chars().next_back();
    !carry.regex_ok
        && matches!(
            prev,
            Some(')' | ']' | '!' | '_' | '0'..='9' | 'A'..='Z' | 'a'..='z')
        )
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

fn adjust_js(ch: char, carry: &mut Carry) {
    // Called only while the current JS frame's depth is above zero.
    let depth = carry.js.last_mut().expect("js frame");
    if ch == '{' {
        *depth += 1;
    } else {
        *depth -= 1;
    }
}

fn adjust_braces(ch: char, carry: &mut Carry) {
    if ch == '{' {
        carry.braces += 1;
    } else {
        carry.braces -= 1;
        if carry.braces == 0 {
            carry.template = true;
        }
    }
}
