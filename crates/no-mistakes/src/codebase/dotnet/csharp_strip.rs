pub(super) fn strip_comments_and_strings(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch == '@' && chars.peek().is_some_and(|(_, next)| *next == '"') {
            chars.next();
            strip_verbatim_string(&mut out, &mut chars);
            continue;
        }
        if ch == '\'' {
            strip_character_literal(&mut out, &mut chars);
            continue;
        }
        if ch == '"' {
            strip_string(&mut out, &mut chars);
            continue;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '/') {
            strip_line_comment(&mut out, &mut chars);
            continue;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '*') {
            strip_block_comment(&mut out, &mut chars);
            continue;
        }
        out.push(ch);
    }
    out
}

fn strip_verbatim_string(
    out: &mut String,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) {
    out.push(' ');
    out.push(' ');
    while let Some((_, ch)) = chars.next() {
        if ch == '"' {
            if chars.peek().is_some_and(|(_, next)| *next == '"') {
                chars.next();
                out.push(' ');
                out.push(' ');
            } else {
                break;
            }
        } else {
            out.push(if ch == '\n' { '\n' } else { ' ' });
        }
    }
}

fn strip_character_literal(
    out: &mut String,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) {
    out.push(' ');
    let mut escaped = false;
    for (_, ch) in chars.by_ref() {
        out.push(' ');
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '\'' {
            break;
        }
    }
}

fn strip_string(out: &mut String, chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) {
    out.push(' ');
    let mut escaped = false;
    for (_, string_ch) in chars.by_ref() {
        out.push(if string_ch == '\n' { '\n' } else { ' ' });
        if escaped {
            escaped = false;
        } else if string_ch == '\\' {
            escaped = true;
        } else if string_ch == '"' {
            break;
        }
    }
}

fn strip_line_comment(
    out: &mut String,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) {
    chars.next();
    out.push(' ');
    out.push(' ');
    for (_, comment_ch) in chars.by_ref() {
        if comment_ch == '\n' {
            out.push('\n');
            break;
        }
        out.push(' ');
    }
}

fn strip_block_comment(
    out: &mut String,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) {
    chars.next();
    out.push(' ');
    out.push(' ');
    let mut previous = '\0';
    for (_, comment_ch) in chars.by_ref() {
        out.push(if comment_ch == '\n' { '\n' } else { ' ' });
        if previous == '*' && comment_ch == '/' {
            break;
        }
        previous = comment_ch;
    }
}
