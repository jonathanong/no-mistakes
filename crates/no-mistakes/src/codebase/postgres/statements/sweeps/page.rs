use super::conjuncts;
use crate::codebase::postgres::idents::{ident_key, object_name_ident, object_name_key};
use crate::codebase::postgres::statements::SqlSweepFact;
use sqlparser::ast::{
    Distinct, Expr, GroupByExpr, OrderBy, OrderByKind, OrderBySort, Query, Select, SelectItem,
    SetExpr, Spanned, TableFactor,
};
use std::collections::BTreeMap;

/// A limited SELECT of one base table, ordered by that table's plain columns: a page of a walk.
/// `ctes` are the CTE names visible to the query.
pub(super) fn sweep(query: &Query, ctes: &[String]) -> Option<SqlSweepFact> {
    let (select, order) = page_of(query)?;
    let [from] = select.from.as_slice() else {
        return None;
    };
    // A TABLESAMPLE restricts the relation before it is ordered: it is not a whole-table walk.
    let TableFactor::Table {
        name,
        alias,
        args: None,
        sample: None,
        ..
    } = &from.relation
    else {
        return None;
    };
    let table = object_name_key(name);
    let grouped = !matches!(
        &select.group_by,
        GroupByExpr::Expressions(expressions, modifiers)
            if expressions.is_empty() && modifiers.is_empty()
    );
    // A one-part name is a CTE reference when a CTE has that name; a quoted dot is no schema.
    let cte = name.0.len() == 1 && ctes.contains(&table);
    let deduplicates = matches!(select.distinct, Some(Distinct::Distinct | Distinct::On(_)));
    if !from.joins.is_empty() || grouped || deduplicates || cte {
        return None;
    }
    // The relation answers to its last name part as written, a quoted dot included.
    let mut names: Vec<String> = object_name_ident(name).map(ident_key).into_iter().collect();
    names.extend(alias.as_ref().map(|alias| ident_key(&alias.name)));
    let order_columns = order_columns(order, select, &names)?;
    let OrderByKind::Expressions(order_expressions) = &order.kind else {
        return None;
    };
    let order_ascending: Vec<_> = order_expressions
        .iter()
        .map(|expression| match expression.options.sort.as_ref() {
            Some(OrderBySort::Asc) | None => Some(true),
            Some(OrderBySort::Desc) => Some(false),
            Some(OrderBySort::Using(_)) => None,
        })
        .collect();
    let conjuncts = conjuncts::of(
        select.selection.as_ref(),
        &names,
        &order_columns,
        &order_ascending,
    );
    let at = name.span().start;
    Some(SqlSweepFact {
        line: (at.line as usize).max(1),
        column: (at.column as usize).max(1),
        table_parts: name
            .0
            .iter()
            .filter_map(|part| part.as_ident())
            .map(ident_key)
            .collect(),
        table,
        order_columns,
        conjuncts,
    })
}

/// The output name PostgreSQL gives an unaliased select item: a column keeps its name and a
/// function call takes the function's (`random()` is `random`, through a cast too).
fn implicit_label(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Nested(inner) => implicit_label(inner),
        Expr::Cast { expr, .. } => implicit_label(expr),
        Expr::Function(function) => object_name_ident(&function.name).map(ident_key),
        Expr::Identifier(ident) => Some(ident_key(ident)),
        Expr::CompoundIdentifier(parts) => parts.last().map(ident_key),
        _ => None,
    }
}

/// The SELECT a query's LIMIT applies to and the ORDER BY that orders its rows, looking through
/// parentheses that carry no WITH or limit of their own. The outermost ORDER BY is the order
/// (`(SELECT …) ORDER BY id LIMIT 5`); when the outer query has none, the inner one orders the
/// rows the outer LIMIT counts (`(SELECT … ORDER BY id) LIMIT 5`).
fn page_of(query: &Query) -> Option<(&Select, &OrderBy)> {
    let mut order = query.order_by.as_ref();
    let mut body = &*query.body;
    while let SetExpr::Query(inner) = body {
        if inner.with.is_some() || inner.limit_clause.is_some() || inner.fetch.is_some() {
            return None;
        }
        order = order.or(inner.order_by.as_ref());
        body = &inner.body;
    }
    match body {
        SetExpr::Select(select) => Some((select, order?)),
        _ => None,
    }
}

/// The ORDER BY columns, all plain columns of the relation. A bare name that is also an output
/// name (an alias, or the label PostgreSQL gives `random()`) means that output expression, so it
/// counts only when that is a plain column.
fn order_columns(order: &OrderBy, select: &Select, names: &[String]) -> Option<Vec<String>> {
    let OrderByKind::Expressions(expressions) = &order.kind else {
        return None;
    };
    let aliases: BTreeMap<String, Option<String>> = select
        .projection
        .iter()
        .filter_map(|item| match item {
            SelectItem::ExprWithAlias { expr, alias } => {
                Some((ident_key(alias), conjuncts::column(expr, names)))
            }
            SelectItem::UnnamedExpr(expr) => {
                implicit_label(expr).map(|label| (label, conjuncts::column(expr, names)))
            }
            _ => None,
        })
        .collect();
    expressions
        .iter()
        .map(|expression| match &expression.expr {
            Expr::Identifier(ident) => match aliases.get(&ident_key(ident)) {
                Some(underlying) => underlying.clone(),
                None => conjuncts::column(&expression.expr, names),
            },
            other => conjuncts::column(other, names),
        })
        .collect()
}
