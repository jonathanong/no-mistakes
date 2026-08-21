pub(super) fn key_line(source: &str, source_key: &str) -> usize {
    let Some((section, rest)) = source_key.split_once('.') else {
        return locate(source, source_key, 0).map_or(1, |(line, _)| line);
    };
    let Some((mut line, mut cursor)) = locate(source, section, 0) else {
        return 1;
    };
    if let Some((found, _)) = locate(source, rest, cursor) {
        return found;
    }
    for segment in rest.split('.') {
        match locate(source, segment, cursor) {
            Some((found, end)) => {
                line = found;
                cursor = end;
            }
            None => break,
        }
    }
    line
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
    let qlen = quote.len_utf8();
    find_key(code, key, |start, after| {
        let quoted = preceding_char(code, start) == Some(quote)
            && code
                .get(after..)
                .is_some_and(|rest| rest.starts_with(quote));
        (quoted && mapping_separator(code.get(after + qlen..)?)).then_some(after + qlen)
    })
}

fn preceding_char(code: &str, index: usize) -> Option<char> {
    code.get(..index)?.chars().next_back()
}

fn bare_end(code: &str, key: &str) -> Option<usize> {
    find_key(code, key, |start, after| {
        let before_ok = start == 0 || !is_key_char(code.as_bytes()[start - 1]);
        (before_ok && mapping_separator(&code[after..])).then_some(after)
    })
}

fn find_key(code: &str, key: &str, ok: impl Fn(usize, usize) -> Option<usize>) -> Option<usize> {
    let mut search = 0;
    while let Some(rel) = code.get(search..)?.find(key) {
        let start = search + rel;
        if let Some(end) = ok(start, start + key.len()) {
            return Some(end);
        }
        search = start + code.get(start..)?.chars().next()?.len_utf8();
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
