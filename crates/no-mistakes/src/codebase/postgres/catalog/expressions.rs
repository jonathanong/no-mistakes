use super::CanonicalOrderKey;
use crate::codebase::postgres::parse_postgres_sql;
use regex::Regex;
use sqlparser::ast::{visit_expressions, Expr, SelectItem, SetExpr, Statement};
use std::ops::ControlFlow;
use std::sync::OnceLock;

pub fn order_prefix_matches(
    actual: &[CanonicalOrderKey],
    expected: &[CanonicalOrderKey],
    ignore_qualifiers: bool,
) -> bool {
    actual.len() >= expected.len()
        && actual.iter().zip(expected).all(|(actual, expected)| {
            expression_matches(&actual.expression, &expected.expression, ignore_qualifiers)
                && actual.ascending == expected.ascending
                && actual.nulls_first == expected.nulls_first
        })
}
pub(super) fn order_prefix_matches_for_qualifiers(
    actual: &[CanonicalOrderKey],
    expected: &[CanonicalOrderKey],
    qualifiers: &[String],
) -> bool {
    actual.len() >= expected.len()
        && actual.iter().zip(expected).all(|(actual, expected)| {
            qualifiers_are_allowed(&actual.expression, qualifiers)
                && expression_matches(&actual.expression, &expected.expression, true)
                && actual.ascending == expected.ascending
                && actual.nulls_first == expected.nulls_first
        })
}
pub fn expression_matches(left: &str, right: &str, ignore_qualifiers: bool) -> bool {
    let left = normalize_expression(left);
    let right = normalize_expression(right);
    if ignore_qualifiers {
        strip_qualifiers(&left) == strip_qualifiers(&right)
    } else {
        left == right
    }
}
pub fn normalize_expression(expression: &str) -> String {
    parse_postgres_expression(expression)
        .map(|expression| normalize_sql_display(&strip_outer_nesting(expression).to_string()))
        .unwrap_or_else(|| normalize_sql_display(expression))
}
pub fn parse_postgres_expression(expression: &str) -> Option<Expr> {
    let mut statements = parse_postgres_sql(&format!("SELECT {expression}")).ok()?;
    let Statement::Query(query) = statements.pop()? else {
        return None;
    };
    let SetExpr::Select(select) = *query.body else {
        return None;
    };
    (select.projection.len() == 1).then_some(())?;
    match select.projection.into_iter().next()? {
        SelectItem::UnnamedExpr(expression)
        | SelectItem::ExprWithAlias {
            expr: expression, ..
        } => Some(expression),
        _ => None,
    }
}
fn strip_outer_nesting(mut expression: Expr) -> Expr {
    while let Expr::Nested(inner) = expression {
        expression = *inner;
    }
    expression
}
fn normalize_sql_display(sql: &str) -> String {
    let mut normalized = String::with_capacity(sql.len());
    let mut quote = None;
    let mut characters = sql.chars().peekable();
    while let Some(character) = characters.next() {
        match quote {
            Some(delimiter) if character == delimiter => {
                normalized.push(character);
                if characters.peek() == Some(&delimiter) {
                    normalized.push(characters.next().expect("peeked quote"));
                } else {
                    quote = None;
                }
            }
            Some(_) => normalized.push(character),
            None if character == '\'' || character == '"' => {
                quote = Some(character);
                normalized.push(character);
            }
            None if character.is_whitespace() => {}
            None => normalized.extend(character.to_lowercase()),
        }
    }
    normalized
}
fn strip_qualifiers(expression: &str) -> String {
    static QUALIFIED_IDENTIFIER: OnceLock<Regex> = OnceLock::new();
    QUALIFIED_IDENTIFIER
        .get_or_init(|| {
            Regex::new(r"(?:[a-z_][a-z0-9_]*\.)+([a-z_][a-z0-9_]*)").expect("valid qualifier regex")
        })
        .replace_all(expression, "$1")
        .into_owned()
}
fn qualifiers_are_allowed(expression: &str, qualifiers: &[String]) -> bool {
    let Some(expression) = parse_postgres_expression(expression) else {
        return false;
    };
    visit_expressions(&expression, |expression| {
        let Expr::CompoundIdentifier(parts) = expression else {
            return ControlFlow::Continue(());
        };
        let qualifier = parts[..parts.len().saturating_sub(1)]
            .iter()
            .map(|part| part.value.as_str())
            .collect::<Vec<_>>()
            .join(".");
        if qualifiers
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&qualifier))
        {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    })
    .is_continue()
}
