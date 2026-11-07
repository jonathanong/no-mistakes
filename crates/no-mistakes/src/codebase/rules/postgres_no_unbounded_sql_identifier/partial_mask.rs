use crate::codebase::postgres::parse_postgres_sql;
use crate::codebase::rules::postgres_sql_cross_file::delimit_identifier_markers;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer};

pub(super) fn mask(sql: &str, bounded: &[bool]) -> Option<String> {
    let sql = delimit_identifier_markers(sql);
    let mut out = String::with_capacity(sql.len());
    let mut rest = sql.as_str();
    let mut count = 0;
    while let Some(at) = rest.find("sql_placeholder_") {
        let after = &rest[at + "sql_placeholder_".len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        let index = after[..digits].parse::<usize>().ok()?.checked_sub(1)?;
        let bounded = bounded.get(index)?;
        if index != count {
            return None;
        }
        let end = at + "sql_placeholder_".len() + digits;
        out.push_str(&rest[..at]);
        let glued = rest[..at].chars().next_back().is_some_and(identifier_char)
            || rest[end..].chars().next().is_some_and(identifier_char);
        if *bounded && !glued && complete_prefix(&out) {
            // Keep marker ordinals in a comment for the original hole alignment.
            out.push_str("/* ");
            out.push_str(&rest[at..end]);
            out.push_str(" */");
        } else {
            out.push_str(&rest[at..end]);
        }
        count += 1;
        rest = &rest[end..];
    }
    out.push_str(rest);
    (count == bounded.len()).then_some(out)
}

fn identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

fn complete_prefix(sql: &str) -> bool {
    let Ok(tokens) = Tokenizer::new(&PostgreSqlDialect {}, sql).tokenize() else {
        return false;
    };
    let tokens: Vec<_> = tokens
        .into_iter()
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect();
    if matches!(tokens.last(), None | Some(Token::LParen | Token::Comma)) {
        return false;
    }
    if parse_postgres_sql(sql).is_ok_and(|statements| !statements.is_empty()) {
        return true;
    }
    let mut openings = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if matches!(token, Token::LParen) {
            openings.push(index);
        } else if matches!(token, Token::RParen) && openings.pop().is_none() {
            return false;
        }
    }
    if openings.is_empty() {
        return false;
    }
    std::iter::once(0)
        .chain(openings.iter().map(|index| index + 1))
        .any(|start| {
            let mut candidate = tokens[start..]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ");
            candidate.extend(std::iter::repeat_n(
                ')',
                openings.iter().filter(|index| **index >= start).count(),
            ));
            parse_postgres_sql(&candidate).is_ok_and(|statements| !statements.is_empty())
        })
}

#[cfg(test)]
#[path = "partial_mask_tests.rs"]
mod tests;
