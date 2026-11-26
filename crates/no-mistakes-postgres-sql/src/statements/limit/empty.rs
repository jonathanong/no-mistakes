use super::numeric_literal;
use sqlparser::ast::{DataType, Expr, LimitClause, Query, UnaryOperator, Value};

/// Semantic zero is independent of whether a count qualifies as a bare literal.
pub(in super::super) fn is_empty_page(query: &Query) -> bool {
    let count = match (&query.fetch, &query.limit_clause) {
        (Some(fetch), _) => fetch.quantity.as_ref(),
        (None, Some(LimitClause::LimitOffset { limit, .. })) => limit.as_ref(),
        _ => None,
    };
    count.is_some_and(zero)
}

fn zero(expr: &Expr) -> bool {
    match expr {
        Expr::Nested(expr)
        | Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            expr,
        } => zero(expr),
        Expr::Cast {
            expr, data_type, ..
        } if numeric_type(data_type) => zero(expr),
        Expr::Value(value) => match &value.value {
            Value::Number(text, _) => {
                numeric_literal(text).is_ok_and(|value| value == 0)
                    || text.split(['e', 'E']).next().is_some_and(|mantissa| {
                        mantissa
                            .bytes()
                            .all(|byte| matches!(byte, b'0' | b'.' | b'_'))
                    })
            }
            _ => false,
        },
        _ => false,
    }
}

/// These PostgreSQL numeric casts preserve zero. User types can define arbitrary casts.
fn numeric_type(data_type: &DataType) -> bool {
    if let DataType::Custom(name, _) = data_type {
        let identifiers: Vec<_> = name.0.iter().filter_map(|part| part.as_ident()).collect();
        return identifiers.len() == 2
            && identifiers.iter().all(|ident| ident.quote_style.is_none())
            && identifiers[0].value.eq_ignore_ascii_case("pg_catalog")
            && ["int2", "int4", "int8", "numeric", "float4", "float8"]
                .iter()
                .any(|name| identifiers[1].value.eq_ignore_ascii_case(name));
    }
    matches!(
        data_type,
        DataType::BigInt(_)
            | DataType::Int(_)
            | DataType::Int2(_)
            | DataType::Int4(_)
            | DataType::Int8(_)
            | DataType::Integer(_)
            | DataType::SmallInt(_)
            | DataType::Numeric(_)
            | DataType::Decimal(_)
            | DataType::Dec(_)
            | DataType::Real
            | DataType::DoublePrecision
            | DataType::Float(_)
            | DataType::Float4
            | DataType::Float8
    )
}
