pub(super) fn parameter_names(line: &str) -> Vec<String> {
    let Some(text) = parameter_text(line) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
            current.push(ch);
        } else if !current.is_empty() {
            names.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        names.push(current);
    }
    names
}

pub(super) fn signature_body(line: &str) -> &str {
    super::syntax::header_paren(line)
        .and_then(|open| close_paren(line, open))
        .map(|close| &line[close..])
        .unwrap_or("")
}

fn parameter_text(line: &str) -> Option<&str> {
    let open = super::syntax::header_paren(line)?;
    let close = close_paren(line, open)?;
    (close > open).then_some(&line[open + 1..close - 1])
}

fn close_paren(line: &str, open: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (offset, ch) in line[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + offset + ch.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}
