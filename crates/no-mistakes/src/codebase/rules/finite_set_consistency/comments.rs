pub(super) fn strip_comments(source: &str) -> String {
    let mut stripped = String::with_capacity(source.len());
    let mut iter = source.char_indices().peekable();
    let mut quote = None;
    let mut escaped = false;
    while let Some((_, ch)) = iter.next() {
        if let Some(active_quote) = quote {
            stripped.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' || ch == '`' {
            quote = Some(ch);
            stripped.push(ch);
            continue;
        }
        if ch == '/' {
            match iter.peek().copied() {
                Some((_, '/')) => {
                    iter.next();
                    for (_, comment_ch) in iter.by_ref() {
                        if comment_ch == '\n' {
                            stripped.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some((_, '*')) => {
                    iter.next();
                    let mut previous = '\0';
                    for (_, comment_ch) in iter.by_ref() {
                        if previous == '*' && comment_ch == '/' {
                            break;
                        }
                        previous = comment_ch;
                    }
                    continue;
                }
                _ => {}
            }
        }
        stripped.push(ch);
    }
    stripped
}
