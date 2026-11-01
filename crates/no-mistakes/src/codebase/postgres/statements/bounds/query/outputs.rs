mod finite;

use super::super::super::value::{is_placeholder_ident_at, PlaceholderPositions};
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::SqlBoundOutput;
use sqlparser::ast::{DataType, Expr, Select, SelectItem, Value};

pub(super) fn select(
    select: &Select,
    items: &[crate::codebase::postgres::SqlBoundItem],
    scope: &super::super::Scope,
    positions: PlaceholderPositions<'_>,
) -> Vec<SqlBoundOutput> {
    let resolver = super::super::pins::Resolver::new(items, scope.names(), positions);
    select
        .projection
        .iter()
        .map(|item| match item {
            SelectItem::ExprWithAlias { expr, alias } => Some(SqlBoundOutput {
                name: Some(ident_key(alias)),
                caller_sized: caller_value(expr, positions),
                scalar: scalar(expr, items, &resolver, positions),
                source_column: resolver.column(expr),
                array_aggregate_column: array_element(expr, select, &resolver),
                constant_null: constant_null(expr),
            }),
            SelectItem::UnnamedExpr(expr) => Some(SqlBoundOutput {
                name: projected_name(expr),
                caller_sized: caller_value(expr, positions),
                scalar: scalar(expr, items, &resolver, positions),
                source_column: resolver.column(expr),
                array_aggregate_column: array_element(expr, select, &resolver),
                constant_null: constant_null(expr),
            }),
            // Wildcards have catalog-dependent width, so later ordinal aliases cannot be mapped.
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default()
}

// A custom cast or function may vary for each duplicate source row. Only direct values and
// trusted builtin scalar casts provide output proof; ordinary row bounds remain separate.
/// A bare identifier is a name. `a.id` stays unnamed, so that projection is not a complete
/// column list and can still hide a bare column of the outer statement.
fn projected_name(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => Some(ident_key(ident)),
        _ => None,
    }
}

fn constant_null(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => matches!(value.value, Value::Null),
        Expr::Nested(inner) | Expr::Cast { expr: inner, .. } => constant_null(inner),
        _ => false,
    }
}

pub(super) fn caller_value(expr: &Expr, positions: PlaceholderPositions<'_>) -> bool {
    match expr {
        Expr::Value(_) => true,
        Expr::Identifier(ident) => is_placeholder_ident_at(ident, positions),
        Expr::Nested(expr) => caller_value(expr, positions),
        Expr::Cast {
            expr,
            data_type:
                DataType::Uuid
                | DataType::Text
                | DataType::Boolean
                | DataType::Bool
                | DataType::Int(_)
                | DataType::Integer(_)
                | DataType::BigInt(_)
                | DataType::SmallInt(_)
                | DataType::Numeric(_)
                | DataType::Decimal(_)
                | DataType::Array(_),
            ..
        } => caller_value(expr, positions),
        _ => false,
    }
}

pub(super) fn common(left: &[SqlBoundOutput], right: &[SqlBoundOutput]) -> Vec<SqlBoundOutput> {
    if left.len() != right.len() {
        return Vec::new();
    }
    left.iter()
        .zip(right)
        .map(|(left, right)| SqlBoundOutput {
            name: left.name.clone(),
            caller_sized: left.caller_sized && right.caller_sized,
            scalar: left.scalar && right.scalar,
            source_column: None,
            array_aggregate_column: None,
            constant_null: left.constant_null && right.constant_null,
        })
        .collect()
}

fn scalar(
    expr: &Expr,
    items: &[crate::codebase::postgres::SqlBoundItem],
    resolver: &super::super::pins::Resolver,
    positions: PlaceholderPositions<'_>,
) -> bool {
    match expr {
        Expr::Nested(expr) => scalar(expr, items, resolver, positions),
        Expr::Value(value) => !matches!(value.value, Value::Placeholder(_)),
        Expr::Cast {
            data_type:
                DataType::Uuid
                | DataType::Text
                | DataType::Boolean
                | DataType::Bool
                | DataType::Int(_)
                | DataType::Integer(_)
                | DataType::BigInt(_)
                | DataType::SmallInt(_)
                | DataType::Numeric(_)
                | DataType::Decimal(_),
            ..
        } => finite::expression(expr, items, resolver, positions),
        _ => false,
    }
}

fn array_element(
    expr: &Expr,
    select: &Select,
    resolver: &super::super::pins::Resolver,
) -> Option<(usize, String)> {
    use sqlparser::ast::{FunctionArg, FunctionArgExpr, FunctionArguments, GroupByExpr};
    if select.having.is_some()
        || !matches!(&select.group_by, GroupByExpr::Expressions(expressions, _) if expressions.is_empty())
    {
        return None;
    }
    let Expr::Function(function) = expr else {
        return None;
    };
    if !super::super::functions::is_aggregate(&function.name)
        || crate::codebase::postgres::idents::object_name_ident(&function.name)
            .is_none_or(|name| crate::codebase::postgres::idents::ident_key(name) != "array_agg")
        || function.over.is_some()
        || function.filter.is_some()
        || !function.within_group.is_empty()
    {
        return None;
    }
    let FunctionArguments::List(arguments) = &function.args else {
        return None;
    };
    if !arguments.clauses.is_empty() {
        return None;
    }
    let [FunctionArg::Unnamed(FunctionArgExpr::Expr(element))] = arguments.args.as_slice() else {
        return None;
    };
    resolver.column(element)
}
