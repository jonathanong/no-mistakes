/// Mask comments, quoted strings, and dollar-quoted bodies in one scan so
/// comment markers inside literals and quotes inside comments cannot nest.
pub fn mask_quoted_sql(sql: &str) -> String {
    mask_comments(sql)
}

/// Same one-pass mask as [`mask_quoted_sql`].
pub fn mask_comments(sql: &str) -> String {
    mask_regions(sql)
}

fn mask_regions(sql: &str) -> String {
    let chars: Vec<char> = sql.chars().collect();
    let mut out = String::with_capacity(sql.len());
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '-' && chars.get(index + 1) == Some(&'-') {
            while index < chars.len() && chars[index] != '\n' {
                out.push(' ');
                index += 1;
            }
            continue;
        }
        if chars[index] == '/' && chars.get(index + 1) == Some(&'*') {
            index = skip_block_comment(&chars, index, &mut out);
            continue;
        }
        if matches!(chars[index], 'e' | 'E') && chars.get(index + 1) == Some(&'\'') {
            out.push(' ');
            index = skip_quote(&chars, index + 1, '\'', &mut out, true);
            continue;
        }
        if chars[index] == '\'' {
            index = skip_quote(&chars, index, '\'', &mut out, false);
            continue;
        }
        if chars[index] == '"' {
            index = skip_quote(&chars, index, '"', &mut out, false);
            continue;
        }
        if chars[index] == '$' {
            if let Some(end) = skip_dollar(&chars, index, &mut out) {
                index = end;
                continue;
            }
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

fn skip_block_comment(chars: &[char], mut index: usize, out: &mut String) -> usize {
    out.push(' ');
    out.push(' ');
    index += 2;
    let mut depth = 1i32;
    while index < chars.len() && depth > 0 {
        if chars[index] == '/' && chars.get(index + 1) == Some(&'*') {
            out.push(' ');
            out.push(' ');
            index += 2;
            depth += 1;
            continue;
        }
        if chars[index] == '*' && chars.get(index + 1) == Some(&'/') {
            out.push(' ');
            out.push(' ');
            index += 2;
            depth -= 1;
            continue;
        }
        out.push(if chars[index] == '\n' { '\n' } else { ' ' });
        index += 1;
    }
    index
}

fn skip_quote(chars: &[char], start: usize, quote: char, out: &mut String, escaped: bool) -> usize {
    out.push(' ');
    let mut index = start + 1;
    while index < chars.len() {
        out.push(' ');
        if escaped && chars[index] == '\\' {
            if index + 1 < chars.len() {
                out.push(' ');
                index += 2;
                continue;
            }
            return chars.len();
        }
        if chars[index] == quote {
            if !escaped && index + 1 < chars.len() && chars[index + 1] == quote {
                out.push(' ');
                index += 2;
                continue;
            }
            return index + 1;
        }
        index += 1;
    }
    index
}

fn skip_dollar(chars: &[char], start: usize, out: &mut String) -> Option<usize> {
    let mut tag = String::from("$");
    let mut index = start + 1;
    while index < chars.len() && (chars[index].is_ascii_alphanumeric() || chars[index] == '_') {
        tag.push(chars[index]);
        index += 1;
    }
    if index >= chars.len() || chars[index] != '$' {
        return None;
    }
    tag.push('$');
    index += 1;
    let tag_chars: Vec<char> = tag.chars().collect();
    while index + tag_chars.len() <= chars.len() {
        if chars[index..index + tag_chars.len()] == tag_chars[..] {
            for _ in 0..tag_chars.len() {
                out.push(' ');
            }
            return Some(index + tag_chars.len());
        }
        out.push(' ');
        index += 1;
    }
    Some(chars.len())
}

pub fn insert_keyword_count(masked: &str) -> usize {
    let lower = masked.to_ascii_lowercase();
    let mut count = 0usize;
    let mut search = lower.as_str();
    while let Some(at) = search.find("insert") {
        let after = &search[at + 6..];
        let trimmed = after.trim_start();
        if trimmed.starts_with("into") {
            count += 1;
        }
        search = &search[at + 6..];
    }
    count
}
