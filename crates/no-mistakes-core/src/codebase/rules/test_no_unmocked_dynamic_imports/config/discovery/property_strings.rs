use regex::Regex;

pub(in crate::codebase::rules::test_no_unmocked_dynamic_imports::config) fn extract_property_strings(
    source: &str,
    property: &str,
) -> Vec<String> {
    let re = Regex::new(&format!(r#"\b{}\s*:\s*"#, regex::escape(property)))
        .expect("property regex compiles");
    let mut strings = Vec::new();
    for mat in re.find_iter(source) {
        let mut idx = mat.end();
        skip_space(source, &mut idx);
        if starts_string(source, idx) {
            if let Some((value, _)) = parse_string(source, idx) {
                strings.push(value);
            }
        } else if source[idx..].starts_with('[') {
            extract_array_strings(source, idx + '['.len_utf8(), &mut strings);
        }
    }
    strings
}

fn extract_array_strings(source: &str, mut idx: usize, strings: &mut Vec<String>) {
    while idx < source.len() {
        skip_space(source, &mut idx);
        if source[idx..].starts_with(']') {
            break;
        }
        if starts_string(source, idx) {
            if let Some((value, end)) = parse_string(source, idx) {
                strings.push(value);
                idx = end;
                continue;
            }
        }
        idx += next_char_len(source, idx);
    }
}

fn parse_string(source: &str, start: usize) -> Option<(String, usize)> {
    let quote = source[start..].chars().next()?;
    let mut value = String::new();
    let mut escaped = false;
    for (offset, ch) in source[start + quote.len_utf8()..].char_indices() {
        let idx = start + quote.len_utf8() + offset;
        if escaped {
            value.push('\\');
            value.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some((value, idx + quote.len_utf8()));
        }
        value.push(ch);
    }
    None
}

fn starts_string(source: &str, idx: usize) -> bool {
    source[idx..].starts_with('\'') || source[idx..].starts_with('"')
}

fn skip_space(source: &str, idx: &mut usize) {
    while *idx < source.len() {
        let ch = source[*idx..]
            .chars()
            .next()
            .expect("idx is within a valid UTF-8 string");
        if !ch.is_whitespace() {
            break;
        }
        *idx += ch.len_utf8();
    }
}

fn next_char_len(source: &str, idx: usize) -> usize {
    source[idx..]
        .chars()
        .next()
        .map(char::len_utf8)
        .unwrap_or(1)
}
