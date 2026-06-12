pub(super) fn body(source: &str) -> &str {
    let mut end = source.len();
    let mut offset = 0usize;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            offset += line.len() + 1;
            continue;
        }
        if offset > 0
            && matches!(
                trimmed.split_whitespace().next(),
                Some(
                    "export"
                        | "import"
                        | "declare"
                        | "const"
                        | "let"
                        | "var"
                        | "type"
                        | "interface"
                        | "class"
                        | "enum"
                        | "function"
                )
            )
        {
            end = offset.saturating_sub(1);
            break;
        }
        if let Some(index) = semicolon_outside_strings(line) {
            end = offset + index;
            break;
        }
        offset += line.len() + 1;
    }
    &source[..end]
}

fn semicolon_outside_strings(line: &str) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
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
        if ch == '"' || ch == '\'' || ch == '`' {
            quote = Some(ch);
            continue;
        }
        if ch == ';' {
            return Some(index);
        }
    }
    None
}
