pub(super) fn key_line(source: &str, source_key: &str) -> usize {
    let mut cursor = 0;
    let mut line = 1;
    for segment in segments(source_key) {
        match locate(source, segment, cursor) {
            Some((found, end)) => {
                line = found;
                cursor = end;
            }
            None => return line,
        }
    }
    line
}

fn segments(key: &str) -> Vec<&str> {
    match key.split_once('.') {
        None => vec![key],
        Some((section, rest)) => {
            let mut parts = vec![section];
            parts.extend(rest.split('.'));
            parts
        }
    }
}

fn locate(source: &str, key: &str, from: usize) -> Option<(usize, usize)> {
    let rest = source.get(from..)?;
    let mut offset = from;
    for piece in rest.split_inclusive('\n') {
        if let Some(end) = key_end(piece, key) {
            let line = source[..offset].bytes().filter(|b| *b == b'\n').count() + 1;
            return Some((line, offset + end));
        }
        offset += piece.len();
    }
    None
}

fn key_end(line: &str, key: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        return None;
    }
    let code = strip_inline_comment(line);
    quoted_end(code, key, '"')
        .or_else(|| quoted_end(code, key, '\''))
        .or_else(|| bare_end(code, key))
        .or_else(|| header_end(code, key))
}

fn strip_inline_comment(line: &str) -> &str {
    let hash = line.find(" #");
    let slashes = line.find(" //");
    match (hash, slashes) {
        (None, None) => line,
        (Some(a), None) => &line[..a],
        (None, Some(b)) => &line[..b],
        (Some(a), Some(b)) => &line[..a.min(b)],
    }
}

fn quoted_end(code: &str, key: &str, quote: char) -> Option<usize> {
    let mut search = 0;
    while let Some(rel) = code[search..].find(key) {
        let start = search + rel;
        let after = start + key.len();
        let quoted = start
            .checked_sub(quote.len_utf8())
            .is_some_and(|i| code[i..].starts_with(quote))
            && code[after..].starts_with(quote);
        if quoted && mapping_separator(&code[after + quote.len_utf8()..]) {
            return Some(after + quote.len_utf8());
        }
        search = start + 1;
    }
    None
}

fn bare_end(code: &str, key: &str) -> Option<usize> {
    let mut search = 0;
    while let Some(rel) = code[search..].find(key) {
        let start = search + rel;
        let after = start + key.len();
        let before_ok = start == 0 || !is_key_char(code.as_bytes()[start - 1]);
        if before_ok && mapping_separator(&code[after..]) {
            return Some(after);
        }
        search = start + 1;
    }
    None
}

fn header_end(code: &str, key: &str) -> Option<usize> {
    header_token_end(code, &format!("[{key}"))
        .or_else(|| header_token_end(code, &format!(".{key}")))
}

fn header_token_end(code: &str, token: &str) -> Option<usize> {
    let start = code.find(token)?;
    let after = start + token.len();
    (code[after..].starts_with(']') || code[after..].starts_with('.')).then_some(after)
}

fn mapping_separator(rest: &str) -> bool {
    let rest = rest.trim_start();
    rest.starts_with(':') || rest.starts_with('=')
}

fn is_key_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}
