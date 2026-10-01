use super::locate::{self, find_opening};
use crate::codebase::postgres::types::SqlDeclaredIdentifier;

/// `sqlparser` rejects PostgreSQL procedure bodies, so the name is read from
/// the source after `CREATE [OR REPLACE] PROCEDURE`.
pub(super) fn procedure_names(sql: &str) -> Vec<SqlDeclaredIdentifier> {
    let mut names = Vec::new();
    let mut from = 0usize;
    while from < sql.len() {
        let Some((start, end)) = next_procedure(&sql[from..]) else {
            break;
        };
        let line = locate::line_number(sql, from + start);
        let Some((name, consumed)) = procedure_ident(&sql[from + end..]) else {
            from += end.max(start + 1);
            continue;
        };
        names.push(SqlDeclaredIdentifier { name, line });
        from += end + consumed;
    }
    names
}

fn next_procedure(sql: &str) -> Option<(usize, usize)> {
    let replaced = find_opening(sql, &["create", "or", "replace", "procedure"]);
    let plain = find_opening(sql, &["create", "procedure"]);
    match (replaced, plain) {
        (Some(replaced), Some(plain)) if replaced.0 <= plain.0 => Some(replaced),
        (Some(replaced), None) => Some(replaced),
        (_, Some(plain)) => Some(plain),
        (None, None) => None,
    }
}

fn procedure_ident(sql: &str) -> Option<(String, usize)> {
    let mut at = 0usize;
    let mut name = read_ident(sql, skip_ws(sql, at))?;
    loop {
        at = skip_ws(sql, name.1);
        if !sql[at..].starts_with('.') {
            return Some((name.0, at));
        }
        name = read_ident(sql, skip_ws(sql, at + 1))?;
    }
}

fn read_ident(sql: &str, start: usize) -> Option<(String, usize)> {
    let bytes = sql.as_bytes();
    if start >= bytes.len() {
        return None;
    }
    if bytes[start] == b'"' {
        return read_quoted(sql, start);
    }
    let rest = &sql[start..];
    let mut len = 0usize;
    for character in rest.chars() {
        if len == 0 && !(character.is_alphabetic() || character == '_' || !character.is_ascii()) {
            return None;
        }
        if len > 0
            && !(character.is_alphanumeric()
                || character == '_'
                || character == '$'
                || !character.is_ascii())
        {
            break;
        }
        len += character.len_utf8();
    }
    (len > 0).then(|| (rest[..len].to_string(), start + len))
}

fn read_quoted(sql: &str, start: usize) -> Option<(String, usize)> {
    let bytes = sql.as_bytes();
    let mut index = start + 1;
    let mut name = String::new();
    while index < bytes.len() {
        if bytes[index] == b'"' {
            if bytes.get(index + 1) == Some(&b'"') {
                name.push('"');
                index += 2;
                continue;
            }
            return Some((name, index + 1));
        }
        let character = sql[index..].chars().next()?;
        name.push(character);
        index += character.len_utf8();
    }
    None
}

fn skip_ws(sql: &str, mut at: usize) -> usize {
    let bytes = sql.as_bytes();
    while at < bytes.len() && bytes[at].is_ascii_whitespace() {
        at += 1;
    }
    at
}
