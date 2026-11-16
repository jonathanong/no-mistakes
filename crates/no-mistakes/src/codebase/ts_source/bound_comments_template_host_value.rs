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
    // A brace outside an interpolation is an operator, same as `(`.
    carry.regex_ok = "([{,;:?=&|*%<>~^/".contains(ch);
    true
}

pub(super) fn of_is_keyword(line: &str, end: usize) -> bool {
    let before = line[..end].trim_end();
    let word_at = before.len().saturating_sub(2);
    for_of_open(line, word_at) && !for_loop_separator(line, end)
}

/// `of` is the keyword when the unmatched `(` belongs to `for` or `for await`
/// and the binding is finished. `=`, `,`, or `;` at depth zero is an initializer.
fn for_of_open(line: &str, word_at: usize) -> bool {
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    for (idx, ch) in line[..word_at].char_indices().rev() {
        match ch {
            ')' => paren += 1,
            ']' => bracket += 1,
            '}' => brace += 1,
            '(' if paren == 0 => return for_before_paren(line, idx),
            '(' => paren -= 1,
            '[' if bracket == 0 => return false,
            '[' => bracket -= 1,
            '{' if brace == 0 => return false,
            '{' => brace -= 1,
            '=' | ',' | ';' if paren == 0 && bracket == 0 && brace == 0 => return false,
            _ => {}
        }
    }
    false
}

fn for_before_paren(line: &str, open_idx: usize) -> bool {
    match super::super::super::super::previous_word(line, open_idx) {
        Some("for") => true,
        Some("await") => {
            let await_at = line[..open_idx]
                .trim_end()
                .len()
                .saturating_sub("await".len());
            super::super::super::super::previous_word(line, await_at) == Some("for")
        }
        _ => false,
    }
}

/// A `for (;;)` header has a `;` before its closing `)`. A for-of regex may contain one.
fn for_loop_separator(line: &str, start: usize) -> bool {
    let tail = &line[start..];
    let trimmed = tail.trim_start();
    let at = line.len() - trimmed.len();
    let scan_from = if trimmed.starts_with('/') {
        regex_close(line, at).map_or(at + 1, |end| end + 1)
    } else {
        start
    };
    semicolon_before_paren(line, scan_from)
}

fn regex_close(line: &str, slash: usize) -> Option<usize> {
    let mut class = false;
    let mut escaped = false;
    let rest = &line[slash + 1..];
    for (offset, ch) in rest.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if class {
            if ch == ']' {
                class = false;
            }
            continue;
        }
        if ch == '[' {
            class = true;
            continue;
        }
        if ch == '/' {
            return Some(slash + 1 + offset);
        }
    }
    None
}

fn semicolon_before_paren(line: &str, start: usize) -> bool {
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    for ch in line[start..].chars() {
        match ch {
            '(' => paren += 1,
            ')' if paren == 0 => return false,
            ')' => paren -= 1,
            '[' => bracket += 1,
            ']' if bracket > 0 => bracket -= 1,
            '{' => brace += 1,
            '}' if brace > 0 => brace -= 1,
            ';' if paren == 0 && bracket == 0 && brace == 0 => return true,
            _ => {}
        }
    }
    false
}
