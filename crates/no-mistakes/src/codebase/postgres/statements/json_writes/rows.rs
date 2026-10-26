use super::{push, SqlJsonWriteFact, SqlWriteColumns};
use sqlparser::ast::{
    Expr, Insert, Select, SelectItem, SetExpr, TableFactor, WildcardAdditionalOptions,
};

pub(super) fn insert(table: &str, insert: &Insert, out: &mut Vec<SqlJsonWriteFact>) {
    let columns: Vec<_> = insert.columns.iter().map(super::column_name).collect();
    if let Some(source) = &insert.source {
        rows(table, &columns, &source.body, out);
    }
}

fn rows(table: &str, columns: &[String], body: &SetExpr, out: &mut Vec<SqlJsonWriteFact>) {
    match body {
        SetExpr::Values(values) => {
            for row in &values.rows {
                row_values(
                    table,
                    columns,
                    row.iter()
                        .enumerate()
                        .map(|(index, expr)| (Some(index), expr))
                        .collect(),
                    Some(row.len()),
                    out,
                );
            }
        }
        SetExpr::Select(select) => {
            if let Some(body) = derived_star(select) {
                rows(table, columns, body, out);
            } else {
                select_values(table, columns, select, out);
            }
        }
        SetExpr::Query(query) => rows(table, columns, &query.body, out),
        SetExpr::SetOperation { left, right, .. } => {
            rows(table, columns, left, out);
            rows(table, columns, right, out);
        }
        _ => {}
    }
}

fn derived_star(select: &Select) -> Option<&SetExpr> {
    let [SelectItem::Wildcard(WildcardAdditionalOptions {
        opt_ilike: None,
        opt_exclude: None,
        opt_except: None,
        opt_replace: None,
        opt_rename: None,
        opt_alias: None,
        ..
    })] = select.projection.as_slice()
    else {
        return None;
    };
    let [from] = select.from.as_slice() else {
        return None;
    };
    if !from.joins.is_empty() {
        return None;
    }
    match &from.relation {
        TableFactor::Derived { subquery, .. } => Some(&subquery.body),
        _ => None,
    }
}

fn select_values(
    table: &str,
    columns: &[String],
    select: &Select,
    out: &mut Vec<SqlJsonWriteFact>,
) {
    let mut next_index = Some(0);
    let mut values = Vec::new();
    for item in &select.projection {
        match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                values.push((next_index, expr));
                next_index = next_index.map(|index| index + 1);
            }
            _ => next_index = None,
        }
    }
    // A wildcard cannot hide an independently known literal JSON expression.
    row_values(table, columns, values, next_index, out);
}

fn row_values(
    table: &str,
    columns: &[String],
    values: Vec<(Option<usize>, &Expr)>,
    width: Option<usize>,
    out: &mut Vec<SqlJsonWriteFact>,
) {
    let written = if columns.is_empty() {
        SqlWriteColumns::Positional(width)
    } else {
        SqlWriteColumns::Named(columns.to_vec())
    };
    for (index, value) in values {
        push(
            table,
            index.and_then(|index| columns.get(index)).cloned(),
            index,
            &written,
            value,
            out,
        );
    }
}
