use crate::codebase::swift::scanner::{find_matching_delimiter, read_quoted_string, Scanner};

pub(super) fn find_package_call(source: &str) -> Option<(usize, usize)> {
    let mut scanner = Scanner::new(source);
    while let Some(index) = scanner.next_code_index() {
        let Some(after) = source[index..].strip_prefix("Package") else {
            continue;
        };
        let prev = source[..index].chars().next_back();
        if prev.is_some_and(|ch| ch == '.' || is_ident_char(ch))
            || after.chars().next().is_some_and(is_ident_char)
        {
            continue;
        }
        let open = next_code_char(source, index + "Package".len())?;
        if open.1 == '(' {
            return find_matching_delimiter(source, open.0, '(', ')').map(|c| (open.0, c));
        }
    }
    None
}

pub(super) fn has_conditional_directive(source: &str) -> bool {
    let mut scanner = Scanner::new(source);
    while let Some(index) = scanner.next_code_index() {
        if source[index..]
            .strip_prefix("#if")
            .is_some_and(|rest| !rest.chars().next().is_some_and(is_ident_char))
        {
            return true;
        }
    }
    false
}

pub(super) fn mutates_field_outside(source: &str, open: usize, close: usize, field: &str) -> bool {
    let mut scanner = Scanner::new(source);
    while let Some(index) = scanner.next_code_index() {
        if index >= open && index <= close {
            scanner.skip_to(close + 1);
            continue;
        }
        if source[index..]
            .strip_prefix('.')
            .and_then(|rest| rest.strip_prefix(field))
            .is_some_and(|after| !after.chars().next().is_some_and(is_ident_char))
        {
            return true;
        }
    }
    false
}

pub(super) fn top_level_label_value<'a>(body: &'a str, label: &str) -> Option<&'a str> {
    let mut scanner = Scanner::new(body);
    let mut depth = 0usize;
    while let Some(index) = scanner.next_code_index() {
        let ch = body[index..].chars().next().unwrap_or('\0');
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 && body[index..].starts_with(label) => {
                let prev = body[..index].chars().next_back();
                let after = index + label.len();
                let next = body[after..].chars().next();
                if prev.is_some_and(is_ident_char) || next.is_some_and(is_ident_char) {
                    continue;
                }
                let Some((colon, ':')) = next_code_char(body, after) else {
                    continue;
                };
                return Some(slice_until_top_level_comma(body, colon + 1));
            }
            _ => {}
        }
    }
    None
}

fn slice_until_top_level_comma(body: &str, start: usize) -> &str {
    let mut scanner = Scanner::new(body);
    scanner.skip_to(start);
    let mut depth = 0usize;
    while let Some(index) = scanner.next_code_index() {
        let ch = body[index..].chars().next().unwrap_or('\0');
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return &body[start..index],
            _ => {}
        }
    }
    &body[start..]
}

pub(super) fn split_top_level_items(body: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut scanner = Scanner::new(body);
    while let Some(index) = scanner.next_code_index() {
        let ch = body[index..].chars().next().unwrap_or('\0');
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                if let Some(trimmed) = trim_code(&body[start..index]) {
                    items.push(trimmed);
                }
                start = index + 1;
            }
            _ => {}
        }
    }
    if let Some(trimmed) = trim_code(&body[start..]) {
        items.push(trimmed);
    }
    items
}

pub(super) fn trim_code(text: &str) -> Option<&str> {
    let mut scanner = Scanner::new(text);
    let mut first = None;
    let mut last_end = 0;
    while let Some(index) = scanner.next_code_index() {
        let ch = text[index..].chars().next().unwrap_or('\0');
        if !ch.is_whitespace() {
            first.get_or_insert(index);
            last_end = if ch == '"' {
                read_quoted_string(text, index).map_or(text.len(), |(_, end)| end)
            } else {
                index + ch.len_utf8()
            };
        }
    }
    first.map(|start| &text[start..last_end])
}

pub(super) fn array_body(expr: &str) -> Option<&str> {
    let expr = trim_code(expr)?;
    let close = expr
        .starts_with('[')
        .then(|| find_matching_delimiter(expr, 0, '[', ']'))
        .flatten()?;
    trim_code(&expr[close + 1..])
        .is_none()
        .then_some(&expr[1..close])
}

pub(super) fn static_string_literal(expr: &str) -> Option<String> {
    let expr = trim_code(expr)?;
    let (value, end) = (expr.starts_with('"') && !expr.starts_with("\"\"\""))
        .then(|| read_quoted_string(expr, 0))
        .flatten()?;
    (trim_code(&expr[end..]).is_none() && !value.contains("\\(") && !value.contains('\n'))
        .then_some(value)
}

pub(super) fn static_string_array(expr: &str) -> Option<Vec<String>> {
    split_top_level_items(array_body(expr)?)
        .into_iter()
        .map(static_string_literal)
        .collect()
}

pub(super) fn call_name_and_args(expr: &str) -> Option<(&str, &str)> {
    let expr = trim_code(expr)?;
    let open = expr.find('(')?;
    let close = find_matching_delimiter(expr, open, '(', ')')?;
    let prefix = trim_code(&expr[close + 1..])
        .is_none()
        .then(|| trim_code(&expr[..open]))
        .flatten()?;
    let name = prefix.rsplit('.').next().unwrap_or("");
    let qualifier = &prefix[..prefix.len() - name.len()];
    matches!(
        qualifier,
        "." | "SupportedPlatform."
            | "PackageDescription.SupportedPlatform."
            | "Target."
            | "PackageDescription.Target."
    )
    .then_some((name, &expr[open + 1..close]))
}

fn next_code_char(source: &str, start: usize) -> Option<(usize, char)> {
    let mut scanner = Scanner::new(source);
    scanner.skip_to(start);
    while let Some(index) = scanner.next_code_index() {
        let ch = source[index..].chars().next().unwrap_or('\0');
        if !ch.is_whitespace() {
            return Some((index, ch));
        }
    }
    None
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
