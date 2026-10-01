use crate::codebase::postgres::parse_postgres_expression;
use sqlparser::ast::Expr;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

pub(super) fn expression(definition: &str) -> Option<Expr> {
    parse_expr(&strip_check(definition))
}

fn strip_check(definition: &str) -> String {
    let trimmed = definition.trim();
    let rest = match trimmed.get(..5) {
        Some(prefix) if prefix.eq_ignore_ascii_case("check") && word_boundary(trimmed) => {
            trimmed[5..].trim_start()
        }
        _ => trimmed,
    };
    unwrap_parens(rest)
}

fn word_boundary(text: &str) -> bool {
    text[5..]
        .chars()
        .next()
        .is_some_and(|character| character.is_whitespace() || character == '(')
}

fn unwrap_parens(sql: &str) -> String {
    let mut current = sql.trim().to_string();
    while wraps(&current) {
        current = current[1..current.len() - 1].trim().to_string();
    }
    current
}

fn wraps(sql: &str) -> bool {
    let bytes = sql.as_bytes();
    if bytes.first() != Some(&b'(') || bytes.last() != Some(&b')') {
        return false;
    }
    let mut depth = 0i32;
    let mut quote = None;
    for (index, byte) in bytes.iter().enumerate() {
        if let Some(mark) = quote {
            if *byte == mark {
                quote = None;
            }
            continue;
        }
        match *byte {
            b'\'' | b'"' => quote = Some(*byte),
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return index + 1 == bytes.len();
                }
            }
            _ => {}
        }
    }
    false
}

fn parse_expr(sql: &str) -> Option<Expr> {
    if let Some(expr) = parse_postgres_expression(sql) {
        return Some(expr);
    }
    Parser::new(&PostgreSqlDialect {})
        .try_with_sql(sql)
        .ok()?
        .parse_expr()
        .ok()
}
