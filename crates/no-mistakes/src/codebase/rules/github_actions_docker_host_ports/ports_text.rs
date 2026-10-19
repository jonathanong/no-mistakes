pub(super) fn inline_specs(rest: &str) -> Vec<String> {
    let rest = strip_comment(rest);
    if let Some(inner) = rest
        .strip_prefix('[')
        .and_then(|text| text.strip_suffix(']'))
    {
        return split_flow(inner);
    }
    if rest.is_empty() || rest == "|" || rest == ">" {
        return Vec::new();
    }
    vec![unquote(rest)]
}

pub(super) fn list_spec(trimmed: &str) -> Option<String> {
    let rest = strip_comment(trimmed.strip_prefix('-')?.trim());
    if rest.is_empty() {
        return None;
    }
    Some(unquote(rest))
}

fn split_flow(inner: &str) -> Vec<String> {
    let mut specs = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    for ch in inner.chars() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            ',' => push_spec(&mut specs, &mut current),
            _ => current.push(ch),
        }
    }
    push_spec(&mut specs, &mut current);
    specs
}

fn push_spec(specs: &mut Vec<String>, current: &mut String) {
    let spec = current.trim().to_string();
    if !spec.is_empty() {
        specs.push(spec);
    }
    current.clear();
}

fn strip_comment(text: &str) -> &str {
    let mut quote = None;
    for (index, ch) in text.char_indices() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '#' if index == 0 || text.as_bytes().get(index - 1) == Some(&b' ') => {
                return text[..index].trim();
            }
            _ => {}
        }
    }
    text.trim()
}

fn unquote(text: &str) -> String {
    let text = text.trim();
    if text.len() >= 2 {
        let bytes = text.as_bytes();
        if (bytes[0] == b'"' || bytes[0] == b'\'') && bytes[0] == bytes[bytes.len() - 1] {
            return text[1..text.len() - 1].to_string();
        }
    }
    text.to_string()
}
