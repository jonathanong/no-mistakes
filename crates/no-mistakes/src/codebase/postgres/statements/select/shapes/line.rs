use sqlparser::ast::Expr;

pub(super) fn line_of(sql: &str, cursor: &mut usize, expr: &Expr) -> usize {
    let snippet = expr.to_string();
    let start = find_snippet(&sql[*cursor..], &snippet)
        .map(|offset| *cursor + offset)
        .or_else(|| find_snippet(sql, &snippet))
        .unwrap_or(0);
    *cursor = start.saturating_add(1);
    sql[..start].bytes().filter(|byte| *byte == b'\n').count() + 1
}

fn find_snippet(haystack: &str, snippet: &str) -> Option<usize> {
    let needle: Vec<char> = snippet
        .chars()
        .filter(|char| !char.is_whitespace())
        .map(|char| char.to_ascii_lowercase())
        .collect();
    if needle.is_empty() {
        return None;
    }
    let chars: Vec<(usize, char)> = haystack.char_indices().collect();
    let mut index = 0usize;
    while index < chars.len() {
        if starts_with(&chars[index..], &needle) {
            return Some(chars[index].0);
        }
        index += 1;
    }
    None
}

fn starts_with(chars: &[(usize, char)], needle: &[char]) -> bool {
    let mut needle_at = 0usize;
    for (_, char) in chars {
        if char.is_whitespace() {
            continue;
        }
        if needle_at >= needle.len() {
            return true;
        }
        if char.to_ascii_lowercase() != needle[needle_at] {
            return false;
        }
        needle_at += 1;
    }
    needle_at == needle.len()
}

#[cfg(test)]
mod tests;
