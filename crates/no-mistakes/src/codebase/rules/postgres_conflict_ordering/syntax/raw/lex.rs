use anyhow::{bail, Result};

pub(super) fn find_keyword(sql: &str, keyword: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    let keyword = keyword.as_bytes();
    let mut index = start;
    let mut quote = None;
    while index < bytes.len() {
        if let Some(delimiter) = quote {
            if bytes[index] == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        match bytes[index] {
            b'\'' | b'"' => quote = Some(bytes[index]),
            b'-' if bytes.get(index + 1) == Some(&b'-') => {
                index = sql[index..]
                    .find('\n')
                    .map_or(bytes.len(), |offset| index + offset);
                continue;
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = sql[index + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |offset| index + offset + 4);
                continue;
            }
            _ => {}
        }
        if index + keyword.len() <= bytes.len()
            && bytes[index..index + keyword.len()].eq_ignore_ascii_case(keyword)
            && !is_identifier_byte(bytes.get(index.wrapping_sub(1)).copied())
            && !is_identifier_byte(bytes.get(index + keyword.len()).copied())
        {
            return Some(index);
        }
        index += 1;
    }
    None
}

pub(super) fn starts_keyword(sql: &str, start: usize, keyword: &str) -> bool {
    let end = start.saturating_add(keyword.len());
    sql.get(start..end)
        .is_some_and(|text| text.eq_ignore_ascii_case(keyword))
        && !is_identifier_byte(sql.as_bytes().get(end).copied())
}

pub(super) fn skip_whitespace(sql: &str, mut index: usize) -> usize {
    while sql
        .as_bytes()
        .get(index)
        .is_some_and(u8::is_ascii_whitespace)
    {
        index += 1;
    }
    index
}

pub(super) fn matching_parenthesis(sql: &str, open: usize) -> Result<usize> {
    let bytes = sql.as_bytes();
    let mut depth = 0usize;
    let mut quote = None;
    for (index, byte) in bytes.iter().copied().enumerate().skip(open) {
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => depth += 1,
            b')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(index);
                }
            }
            _ => {}
        }
    }
    bail!("unclosed ON CONFLICT target")
}

pub(super) fn split_top_level(sql: &str) -> Vec<String> {
    let mut expressions = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    for (index, byte) in sql.as_bytes().iter().copied().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                expressions.push(sql[start..index].trim().to_string());
                start = index + 1;
            }
            _ => {}
        }
    }
    let final_expression = sql[start..].trim();
    if !final_expression.is_empty() {
        expressions.push(final_expression.to_string());
    }
    expressions
}

pub(super) fn take_identifier(sql: &str, start: usize) -> Result<(String, usize)> {
    let bytes = sql.as_bytes();
    if bytes.get(start) == Some(&b'"') {
        let end = sql[start + 1..]
            .find('"')
            .map(|offset| start + offset + 2)
            .ok_or_else(|| anyhow::anyhow!("unclosed constraint identifier"))?;
        return Ok((sql[start..end].to_string(), end));
    }
    let end = bytes[start..]
        .iter()
        .position(|byte| !is_identifier_byte(Some(*byte)) && *byte != b'.')
        .map_or(bytes.len(), |offset| start + offset);
    if end == start {
        bail!("ON CONFLICT ON CONSTRAINT has no constraint name");
    }
    Ok((sql[start..end].to_string(), end))
}

fn is_identifier_byte(byte: Option<u8>) -> bool {
    byte.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
