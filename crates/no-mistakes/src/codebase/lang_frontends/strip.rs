/// Strip comments while keeping string literals so route/queue names survive.
pub(crate) fn strip_comments_keep_strings(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '#' && chars.peek() != Some(&'[') {
            skip_line(&mut chars);
            out.push('\n');
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'/') {
            chars.next();
            skip_line(&mut chars);
            out.push('\n');
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            skip_block(&mut chars, &mut out);
            continue;
        }
        if ch == '"' || ch == '\'' {
            out.push(ch);
            copy_quoted(&mut chars, &mut out, ch);
            continue;
        }
        out.push(ch);
    }
    out
}

fn skip_line(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    for ch in chars.by_ref() {
        if ch == '\n' {
            break;
        }
    }
}

fn skip_block(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, out: &mut String) {
    let mut previous = '\0';
    for ch in chars.by_ref() {
        if ch == '\n' {
            out.push('\n');
        }
        if previous == '*' && ch == '/' {
            break;
        }
        previous = ch;
    }
}

fn copy_quoted(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    out: &mut String,
    quote: char,
) {
    let mut escaped = false;
    for ch in chars.by_ref() {
        out.push(ch);
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            break;
        }
    }
}

/// Replace triple-quoted contents so docstring examples are not routes.
pub(crate) fn mask_triple_quoted_strings(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if (ch == '"' || ch == '\'')
            && chars.get(i + 1) == Some(&ch)
            && chars.get(i + 2) == Some(&ch)
        {
            i = mask_quoted(&chars, &mut out, i, ch);
            continue;
        }
        out.push(ch);
        i += 1;
    }
    out
}

/// Replace string contents with spaces so docstring examples are not symbols.
pub(crate) fn mask_strings(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' || ch == '\'' || ch == '`' {
            i = mask_quoted(&chars, &mut out, i, ch);
            continue;
        }
        out.push(ch);
        i += 1;
    }
    out
}

fn mask_quoted(chars: &[char], out: &mut String, start: usize, quote: char) -> usize {
    let triple = chars.get(start + 1) == Some(&quote) && chars.get(start + 2) == Some(&quote);
    let width = if triple { 3 } else { 1 };
    for _ in 0..width {
        out.push(quote);
    }
    let mut i = start + width;
    let mut escaped = false;
    while i < chars.len() {
        if !triple && escaped {
            push_masked(out, chars[i]);
            escaped = false;
            i += 1;
            continue;
        }
        if !triple && chars[i] == '\\' {
            out.push(' ');
            escaped = true;
            i += 1;
            continue;
        }
        if triple && chars.get(i..i + 3) == Some(&[quote, quote, quote]) {
            out.push(quote);
            out.push(quote);
            out.push(quote);
            return i + 3;
        }
        if !triple && chars[i] == quote {
            out.push(quote);
            return i + 1;
        }
        push_masked(out, chars[i]);
        i += 1;
    }
    i
}

fn push_masked(out: &mut String, ch: char) {
    if ch == '\n' {
        out.push('\n');
    } else {
        out.push(' ');
    }
}
