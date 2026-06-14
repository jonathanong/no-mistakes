pub(super) fn assignment_index(source: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    let mut iter = source
        .char_indices()
        .skip_while(|(idx, _)| *idx < start)
        .peekable();
    while let Some((idx, ch)) = iter.next() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '=' if iter.peek().is_none_or(|(_, next)| *next != '>') => return Some(idx + 1),
            _ => {}
        }
    }
    None
}

pub(in crate::codebase::rules::finite_set_consistency) fn matching_brace(
    source: &str,
    open: usize,
) -> Option<usize> {
    matching_delimiter(source, open, '{', '}')
}

pub(in crate::codebase::rules::finite_set_consistency) fn matching_delimiter(
    source: &str,
    open: usize,
    open_ch: char,
    close_ch: char,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut previous_significant = None;
    let mut iter = source
        .char_indices()
        .skip_while(|(idx, _)| *idx < open)
        .peekable();
    while let Some((idx, ch)) = iter.next() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }
        if ch == '/' {
            match iter.peek().copied() {
                Some((_, '/')) => {
                    iter.next();
                    for (_, comment_ch) in iter.by_ref() {
                        if comment_ch == '\n' {
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
                _ if regex_can_start_after(previous_significant) => {
                    skip_regex_literal(&mut iter);
                    previous_significant = Some('/');
                    continue;
                }
                _ => {}
            }
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            ch if ch == open_ch => depth += 1,
            ch if ch == close_ch => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
        if !ch.is_whitespace() {
            previous_significant = Some(ch);
        }
    }
    None
}

pub(in crate::codebase::rules::finite_set_consistency) fn top_level_value_end(
    source: &str,
) -> usize {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut previous_significant = None;
    let mut iter = source.char_indices().peekable();
    while let Some((idx, ch)) = iter.next() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }
        if ch == '/' && regex_can_start_after(previous_significant) {
            match iter.peek().copied() {
                Some((_, '/')) | Some((_, '*')) => {}
                _ => {
                    skip_regex_literal(&mut iter);
                    previous_significant = Some('/');
                    continue;
                }
            }
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return idx,
            _ => {}
        }
        if !ch.is_whitespace() {
            previous_significant = Some(ch);
        }
    }
    source.len()
}

fn regex_can_start_after(previous: Option<char>) -> bool {
    previous.is_none_or(|ch| matches!(ch, '(' | '[' | '{' | ',' | ':' | '=' | '!' | '?' | ';'))
}

fn skip_regex_literal<I>(iter: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = (usize, char)>,
{
    let mut escaped = false;
    let mut char_class = false;
    for (_, ch) in iter.by_ref() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '[' => char_class = true,
            ']' => char_class = false,
            '/' if !char_class => break,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
