//! Closed inclusive domains from checks and query predicates.
mod dnf;
mod parse;
use crate::codebase::postgres::canonical_bound_expr;
use sqlparser::ast::{Expr, TableConstraint};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Token;

#[derive(Clone, Copy)]
pub(super) struct Span {
    low: i64,
    high: i64,
}

impl Span {
    pub(super) fn width(self) -> Option<u64> {
        let count = self.high.checked_sub(self.low)?.checked_add(1)?;
        u64::try_from(count).ok()
    }

    fn intersect(self, other: Self) -> Option<Self> {
        let low = self.low.max(other.low);
        let high = self.high.min(other.high);
        (low <= high).then_some(Self { low, high })
    }
}

#[derive(Clone, Copy)]
pub(super) enum Literal {
    Integer,
    Date,
}

pub(super) fn integer_type(data_type: &str) -> bool {
    matches!(
        data_type.trim().to_ascii_lowercase().as_str(),
        "smallint" | "int2" | "integer" | "int" | "int4" | "bigint" | "int8"
    )
}

/// `qualifiers` are `table` and `schema.table` names that may prefix `column`.
pub(super) fn owned_check(definition: &str, column: &str, qualifiers: &[String]) -> Option<Span> {
    let expr = check_expr(definition)?;
    if dnf::contains_or(&expr) || dnf::negated_between(&expr, column, qualifiers) {
        return None;
    }
    let canonical = canonical_bound_expr(&expr);
    let mut conjuncts = Vec::new();
    dnf::and_conjuncts(&canonical, &mut conjuncts);
    span_of(column, &conjuncts, Literal::Integer, qualifiers)
}

pub(super) fn intersect(left: Option<Span>, right: Option<Span>) -> Option<Span> {
    match (left, right) {
        (Some(left), Some(right)) => left.intersect(right),
        (Some(span), None) | (None, Some(span)) => Some(span),
        (None, None) => None,
    }
}

pub(super) fn query_spans(conjuncts: &[Expr], alternatives: usize) -> Vec<Vec<Expr>> {
    dnf::query_spans(conjuncts, alternatives)
}

pub(super) fn closed_span(column: &str, conjuncts: &[Expr], kind: Literal) -> Option<Span> {
    span_of(column, conjuncts, kind, &[])
}

fn span_of(column: &str, conjuncts: &[Expr], kind: Literal, qualifiers: &[String]) -> Option<Span> {
    let mut low = None;
    let mut high = None;
    for conjunct in conjuncts {
        parse::absorb(column, conjunct, kind, &mut low, &mut high, qualifiers);
    }
    let span = Span {
        low: low?,
        high: high?,
    };
    (span.low <= span.high).then_some(span)
}

/// Days since 1970-01-01. Invalid calendar days and years outside 1..=9999 are absent.
pub(super) fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
        return None;
    }
    let max_day = if matches!(month, 1 | 3 | 5 | 7 | 8 | 10 | 12) {
        31
    } else if month == 2 {
        28 + u32::from(year % 4 == 0 && (year % 100 != 0 || year % 400 == 0))
    } else {
        30
    };
    if day == 0 || day > max_day {
        return None;
    }
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(i64::from(era) * 146097 + i64::from(doe) - 719468)
}

fn check_expr(definition: &str) -> Option<Expr> {
    let mut parser = Parser::new(&PostgreSqlDialect {})
        .try_with_sql(definition)
        .ok()?;
    let TableConstraint::Check(check) = parser.parse_optional_table_constraint().ok()?? else {
        return None;
    };
    (parser.peek_token().token == Token::EOF && check.enforced != Some(false))
        .then_some(*check.expr)
}

#[cfg(test)]
mod qualified_tests;
#[cfg(test)]
mod tests;
