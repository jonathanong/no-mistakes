/// Strip comments while keeping string literals so route/queue names survive.
pub(crate) fn strip_comments_keep_strings(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '#' {
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

fn copy_quoted(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, out: &mut String, quote: char) {
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
