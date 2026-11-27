pub(super) fn split_args(inner: &str) -> Option<Vec<&str>> {
    let mut args = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    let mut quote: Option<char> = None;
    for (index, ch) in inner.char_indices() {
        if let Some(open) = quote {
            if ch == open {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
            }
            ',' if depth == 0 => {
                push_arg(&mut args, inner, start, index);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if quote.is_some() || depth != 0 {
        return None;
    }
    push_arg(&mut args, inner, start, inner.len());
    Some(args)
}

pub(super) fn matching_paren(source: &str, args_at: usize) -> Option<usize> {
    scan_until(source, args_at, true)
}

pub(super) fn statement_tail(source: &str, start: usize) -> &str {
    if start > source.len() {
        return "";
    }
    match scan_until(source, start, false) {
        Some(end) => &source[start..end],
        None => source.get(start..).unwrap_or(""),
    }
}

fn push_arg<'a>(args: &mut Vec<&'a str>, inner: &'a str, start: usize, end: usize) {
    let arg = inner[start..end].trim();
    if !arg.is_empty() {
        args.push(arg);
    }
}

fn scan_until(source: &str, start: usize, paren: bool) -> Option<usize> {
    let mut depth = usize::from(paren);
    let mut quote: Option<char> = None;
    for (index, ch) in source[start..].char_indices() {
        if let Some(open) = quote {
            if ch == open {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if paren && depth == 0 {
                    return Some(start + index);
                }
            }
            ';' if !paren && depth == 0 => return Some(start + index),
            _ => {}
        }
    }
    None
}
