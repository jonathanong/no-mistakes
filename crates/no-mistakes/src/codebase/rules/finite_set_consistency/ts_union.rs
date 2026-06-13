pub(super) fn body(source: &str) -> &str {
    let mut end = source.len();
    let mut offset = 0usize;
    for chunk in source.split_inclusive('\n') {
        let line = chunk.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            offset += chunk.len();
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
            end = previous_line_end(source, offset);
            break;
        }
        if let Some(index) = semicolon_outside_strings(line) {
            end = offset + index;
            break;
        }
        offset += chunk.len();
    }
    &source[..end]
}

fn previous_line_end(source: &str, offset: usize) -> usize {
    source[..offset].trim_end_matches(['\r', '\n']).len()
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
