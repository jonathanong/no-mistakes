fn leading_comment_text(line: &str) -> Option<&str> {
    line.strip_prefix("//")
        .or_else(|| line.strip_prefix('#'))
        .or_else(|| line.strip_prefix("--"))
        .map(str::trim)
}

fn line_comment_start(line: &str, mut in_block_comment: bool) -> (Option<(usize, usize)>, bool) {
    let mut quote = None;
    let mut escaped = false;
    let mut in_regex = false;
    let mut in_regex_char_class = false;
    let mut prev_significant = None;
    let mut chars = line.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if in_block_comment {
            if ch == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }
        if in_regex {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '[' {
                in_regex_char_class = true;
            } else if ch == ']' {
                in_regex_char_class = false;
            } else if ch == '/' && !in_regex_char_class {
                in_regex = false;
                prev_significant = Some('/');
            }
            continue;
        }
        if let Some(current) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == current {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '\'' | '"' | '`') {
            quote = Some(ch);
            continue;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '*') {
            chars.next();
            in_block_comment = true;
            continue;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '/') {
            return (Some((idx, 2)), in_block_comment);
        }
        if ch == '/' && regex_can_start_after(prev_significant) {
            in_regex = true;
            continue;
        }
        if ch == '#' {
            return (Some((idx, 1)), in_block_comment);
        }
        if ch == '-' && chars.peek().is_some_and(|(_, next)| *next == '-') {
            return (Some((idx, 2)), in_block_comment);
        }
        if !ch.is_whitespace() {
            prev_significant = Some(ch);
        }
    }
    (None, in_block_comment)
}

fn regex_can_start_after(prev: Option<char>) -> bool {
    prev.is_none_or(|ch| matches!(ch, '(' | '[' | '{' | '=' | ':' | ',' | ';' | '!' | '?'))
}
