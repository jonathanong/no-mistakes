use crate::codebase::postgres::idents::{ident_key, object_name_key, unwrap_expr};
use crate::codebase::postgres::statements::value::{from_expr_at, PlaceholderPositions};
use crate::codebase::postgres::SqlValueForm;
use sqlparser::ast::{BinaryOperator, Expr, LimitClause, Query, SetExpr, TableFactor, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect(
    query: &Query,
    ctes: &[String],
    positions: PlaceholderPositions<'_>,
    out: &mut BTreeMap<(usize, usize), Vec<String>>,
) {
    let Some(LimitClause::LimitOffset {
        limit: Some(limit), ..
    }) = &query.limit_clause
    else {
        return;
    };
    if !count(limit, positions) {
        return;
    }
    let SetExpr::Select(select) = &*query.body else {
        return;
    };
    let [from] = select.from.as_slice() else {
        return;
    };
    let TableFactor::Table {
        name,
        alias,
        args: None,
        ..
    } = &from.relation
    else {
        return;
    };
    let table = object_name_key(name);
    if !from.joins.is_empty()
        || (name.0.len() == 1 && ctes.contains(&table))
        || alias
            .as_ref()
            .is_some_and(|alias| !alias.columns.is_empty())
    {
        return;
    }
    let qualifier = alias
        .as_ref()
        .map(|alias| ident_key(&alias.name))
        .or_else(|| crate::codebase::postgres::idents::object_name_ident(name).map(ident_key));
    let Some(selection) = &select.selection else {
        return;
    };
    let columns = equalities(selection, qualifier.as_deref(), positions);
    if !columns.is_empty() {
        let at = select.select_token.0.span.start;
        out.insert(
            (at.line as usize, at.column as usize),
            columns.into_iter().collect(),
        );
    }
}

fn count(expr: &Expr, positions: PlaceholderPositions<'_>) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => match &value.value {
            Value::Number(text, _) => text.parse::<u64>().is_ok(),
            Value::Placeholder(_) => true,
            _ => false,
        },
        Expr::Identifier(ident) => super::super::value::is_placeholder_ident_at(ident, positions),
        Expr::Cast { expr, .. } => count(expr, positions),
        _ => false,
    }
}

fn equalities(
    expr: &Expr,
    qualifier: Option<&str>,
    positions: PlaceholderPositions<'_>,
) -> BTreeSet<String> {
    let Expr::BinaryOp { left, op, right } = unwrap_expr(expr) else {
        return BTreeSet::new();
    };
    match op {
        BinaryOperator::And => {
            let mut columns = equalities(left, qualifier, positions);
            columns.extend(equalities(right, qualifier, positions));
            columns
        }
        BinaryOperator::Or => equalities(left, qualifier, positions)
            .intersection(&equalities(right, qualifier, positions))
            .cloned()
            .collect(),
        BinaryOperator::Eq => [
            (left.as_ref(), right.as_ref()),
            (right.as_ref(), left.as_ref()),
        ]
        .into_iter()
        .filter(|(_, value)| scalar_value(value, positions))
        .filter_map(|(column, _)| column_name(column, qualifier, positions))
        .collect(),
        _ => BTreeSet::new(),
    }
}

fn column_name(
    expr: &Expr,
    qualifier: Option<&str>,
    positions: PlaceholderPositions<'_>,
) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident)
            if !super::super::value::is_placeholder_ident_at(ident, positions) =>
        {
            Some(ident_key(ident))
        }
        Expr::CompoundIdentifier(parts)
            if parts.len() == 2 && qualifier == Some(ident_key(&parts[0]).as_str()) =>
        {
            Some(ident_key(&parts[1]))
        }
        _ => None,
    }
}

fn scalar_value(expr: &Expr, positions: PlaceholderPositions<'_>) -> bool {
    match unwrap_expr(expr) {
        Expr::Cast { expr, .. } => scalar_value(expr, positions),
        other => matches!(
            from_expr_at(other, positions),
            SqlValueForm::Literal | SqlValueForm::Placeholder
        ),
    }
}
