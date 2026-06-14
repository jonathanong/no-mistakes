pub fn count_code_lines(source: &str) -> usize {
    let mut count = 0;
    let mut block_depth: usize = 0;
    let mut in_string = false;
    let mut in_raw_string: Option<usize> = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Char literals and escape state reset at each line boundary.
        let mut in_char = false;
        let mut escape = false;
        let bytes = trimmed.as_bytes();
        let mut i = 0;
        let mut is_code = false;
        while i < bytes.len() {
            let b = bytes[i];
            if escape {
                escape = false;
                is_code = true;
                i += 1;
                continue;
            }
            if let Some(hash_count) = in_raw_string {
                is_code = true;
                if raw_string_ends_at(bytes, i, hash_count) {
                    in_raw_string = None;
                    i += 1 + hash_count;
                } else {
                    i += 1;
                }
                continue;
            }
            if in_string {
                is_code = true;
                if b == b'\\' {
                    escape = true;
                } else if b == b'"' {
                    in_string = false;
                }
                i += 1;
                continue;
            }
            if in_char {
                is_code = true;
                if b == b'\\' {
                    escape = true;
                } else if b == b'\'' {
                    in_char = false;
                }
                i += 1;
                continue;
            }
            if block_depth > 0 {
                if i + 1 < bytes.len() && b == b'*' && bytes[i + 1] == b'/' {
                    block_depth -= 1;
                    i += 2;
                } else if i + 1 < bytes.len() && b == b'/' && bytes[i + 1] == b'*' {
                    // Rust supports nested block comments.
                    block_depth += 1;
                    i += 2;
                } else {
                    i += 1;
                }
            } else if let Some((hash_count, len)) = raw_string_starts_at(bytes, i) {
                in_raw_string = Some(hash_count);
                is_code = true;
                i += len;
            } else if b == b'"' {
                in_string = true;
                is_code = true;
                i += 1;
            } else if b == b'\'' {
                in_char = true;
                is_code = true;
                i += 1;
            } else if i + 1 < bytes.len() && b == b'/' && bytes[i + 1] == b'*' {
                block_depth += 1;
                i += 2;
            } else if i + 1 < bytes.len() && b == b'/' && bytes[i + 1] == b'/' {
                break;
            } else {
                is_code = true;
                i += 1;
            }
        }
        if is_code {
            count += 1;
        }
    }
    count
}

fn raw_string_starts_at(bytes: &[u8], i: usize) -> Option<(usize, usize)> {
    if bytes.get(i) == Some(&b'r') {
        raw_string_hashes(bytes, i + 1).map(|hash_count| (hash_count, 1 + hash_count + 1))
    } else if bytes.get(i) == Some(&b'b') && bytes.get(i + 1) == Some(&b'r') {
        raw_string_hashes(bytes, i + 2).map(|hash_count| (hash_count, 2 + hash_count + 1))
    } else {
        None
    }
}

fn raw_string_hashes(bytes: &[u8], i: usize) -> Option<usize> {
    let hash_count = bytes.get(i..)?.iter().take_while(|&&b| b == b'#').count();
    (bytes.get(i + hash_count) == Some(&b'"')).then_some(hash_count)
}

fn raw_string_ends_at(bytes: &[u8], i: usize, hash_count: usize) -> bool {
    bytes.get(i) == Some(&b'"')
        && (0..hash_count).all(|offset| bytes.get(i + 1 + offset) == Some(&b'#'))
}
