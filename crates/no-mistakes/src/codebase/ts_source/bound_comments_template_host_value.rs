use super::super::Carry;

pub(super) fn value_step(
    ch: char,
    next: Option<char>,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    carry: &mut Carry,
    ident: &mut String,
) -> bool {
    // `++` and a single `+` or `-` may precede a regex. Postfix `--` does not.
    if ch == '+' || ch == '-' {
        if next == Some(ch) {
            chars.next();
            carry.regex_ok = false;
        } else {
            carry.regex_ok = true;
        }
        return true;
    }
    // Prefix `!` keeps regex_ok; a non-null assertion after a value leaves it false.
    if ch == '!' {
        return true;
    }
    if ch == '\'' || ch == '"' {
        carry.quote = Some(ch);
        carry.escaped = false;
        carry.regex_ok = false;
        return true;
    }
    if ch == '`' {
        open_template(carry);
        return true;
    }
    if super::is_ident(ch) {
        ident.push(ch);
        carry.regex_ok = false;
        return true;
    }
    if ch == '{' || ch == '}' {
        return brace_step(ch, carry);
    }
    if !ch.is_whitespace() {
        carry.regex_ok = "([{,;:?=&|*%<>~^/".contains(ch);
    }
    true
}

fn open_template(carry: &mut Carry) {
    let inside = carry.braces > 0 || carry.js.last().is_some_and(|depth| *depth > 0);
    if inside {
        carry.js.push(0);
    } else {
        carry.template = true;
    }
    carry.regex_ok = false;
}

fn brace_step(ch: char, carry: &mut Carry) -> bool {
    if carry.js.last().is_some_and(|depth| *depth > 0) {
        super::adjust_js(ch, carry);
        carry.regex_ok = ch == '{';
        return true;
    }
    if carry.braces > 0 {
        super::adjust_braces(ch, carry);
        carry.regex_ok = ch == '{';
        return true;
    }
    if !ch.is_whitespace() {
        carry.regex_ok = "([{,;:?=&|*%<>~^/".contains(ch);
    }
    true
}
