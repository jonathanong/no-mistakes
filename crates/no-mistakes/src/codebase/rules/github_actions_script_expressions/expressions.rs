pub(super) fn expressions(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut found = Vec::new();
    while index + 2 < bytes.len() {
        if bytes[index] == b'$' && bytes[index + 1] == b'{' && bytes[index + 2] == b'{' {
            let inner = index + 3;
            if let Some(end) = expression_end(bytes, inner) {
                found.push(text[inner..end].trim().to_string());
                index = end + 2;
                continue;
            }
        }
        index += 1;
    }
    found
}

fn expression_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start;
    let mut quote: Option<u8> = None;
    let mut escaped = false;
    while index < bytes.len() {
        let ch = bytes[index];
        if let Some(active) = quote {
            if escaped {
                escaped = false;
            } else if ch == b'\\' {
                escaped = true;
            } else if ch == active {
                quote = None;
            }
            index += 1;
            continue;
        }
        if ch == b'\'' || ch == b'"' {
            quote = Some(ch);
            index += 1;
            continue;
        }
        if ch == b'}' && bytes.get(index + 1) == Some(&b'}') {
            return Some(index);
        }
        index += 1;
    }
    None
}
