/// `Topic` fields at brace depth 1 inside `Writer` or `ProducerMessage`.
pub(super) fn go_outer_topic_spans(source: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < source.len() {
        let ch = source[i..].chars().next().unwrap();
        if ch == '"' || ch == '\'' || ch == '`' {
            i += ch.len_utf8();
            skip_go_string(source, &mut i, ch);
            continue;
        }
        if let Some(brace) = go_record_brace(source, i) {
            i = brace + 1;
            collect_depth_one_topics(source, &mut i, &mut spans);
            continue;
        }
        i += ch.len_utf8();
    }
    spans
}

fn go_record_brace(source: &str, i: usize) -> Option<usize> {
    if !ident_start(source, i) {
        return None;
    }
    let rest = &source[i..];
    let name_len = if rest.starts_with("ProducerMessage") {
        "ProducerMessage".len()
    } else if rest.starts_with("Writer") {
        "Writer".len()
    } else {
        return None;
    };
    let after_name = &rest[name_len..];
    let trimmed = after_name.trim_start();
    if !trimmed.starts_with('{') {
        return None;
    }
    Some(i + name_len + (after_name.len() - trimmed.len()))
}

fn collect_depth_one_topics(source: &str, i: &mut usize, spans: &mut Vec<(usize, usize)>) {
    let mut depth = 1u32;
    while *i < source.len() && depth > 0 {
        let ch = source[*i..].chars().next().unwrap();
        let len = ch.len_utf8();
        if ch == '"' || ch == '\'' || ch == '`' {
            *i += len;
            skip_go_string(source, i, ch);
            continue;
        }
        if ch == '{' {
            depth += 1;
            *i += len;
            continue;
        }
        if ch == '}' {
            depth -= 1;
            *i += len;
            continue;
        }
        if depth == 1 {
            if let Some((start, end, next)) = topic_literal_span(source, *i) {
                spans.push((start, end));
                *i = next;
                continue;
            }
        }
        *i += len;
    }
}

fn topic_literal_span(source: &str, i: usize) -> Option<(usize, usize, usize)> {
    let rest = source.get(i..)?;
    if !rest.starts_with("Topic") || !topic_boundary(source, i) {
        return None;
    }
    let mut cursor = i + "Topic".len();
    cursor = skip_ws(source, cursor);
    if !source.get(cursor..)?.starts_with(':') {
        return None;
    }
    cursor = skip_ws(source, cursor + 1);
    let quote = source.get(cursor..)?.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    cursor += quote.len_utf8();
    let body = source.get(cursor..)?;
    let close = body.find(quote)?;
    let text = &body[..close];
    if text.is_empty() || text.contains('\n') || text.contains('\r') {
        return None;
    }
    let end = cursor + close;
    Some((cursor, end, end + quote.len_utf8()))
}

fn ident_start(source: &str, i: usize) -> bool {
    source[..i]
        .chars()
        .next_back()
        .is_none_or(|prev| !prev.is_ascii_alphanumeric() && prev != '_')
}

fn topic_boundary(source: &str, i: usize) -> bool {
    ident_start(source, i)
        && !matches!(
            source[i + "Topic".len()..].chars().next(),
            Some(ch) if ch.is_ascii_alphanumeric() || ch == '_'
        )
}

fn skip_ws(source: &str, mut i: usize) -> usize {
    while let Some(ch) = source[i..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        i += ch.len_utf8();
    }
    i
}

fn skip_go_string(source: &str, i: &mut usize, quote: char) {
    let rest = &source[*i..];
    if quote == '`' {
        *i += rest.find('`').map(|end| end + 1).unwrap_or(rest.len());
        return;
    }
    let mut escaped = false;
    for (offset, ch) in rest.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            *i += offset + ch.len_utf8();
            return;
        }
    }
    *i = source.len();
}
