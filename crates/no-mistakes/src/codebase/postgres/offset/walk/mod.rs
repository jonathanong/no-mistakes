mod expr;

use super::OffsetUse;
use expr::expr_offsets as walk_expr;
use sqlparser::ast::{
    Distinct, Expr, GroupByExpr, LimitClause, NamedWindowDefinition, NamedWindowExpr, Query,
    Select, SelectItem, SetExpr, Value, Values, WindowSpec,
};

pub(super) fn query_offsets(query: &Query, out: &mut Vec<OffsetUse>) {
    limit_offsets(query.limit_clause.as_ref(), out);
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            query_offsets(&cte.query, out);
        }
    }
    if let Some(order_by) = &query.order_by {
        if let sqlparser::ast::OrderByKind::Expressions(exprs) = &order_by.kind {
            for item in exprs {
                expr_offsets(&item.expr, out);
            }
        }
    }
    set_expr_offsets(&query.body, out);
}

fn limit_offsets(clause: Option<&LimitClause>, out: &mut Vec<OffsetUse>) {
    match clause {
        Some(LimitClause::LimitOffset {
            limit,
            offset,
            limit_by,
        }) => {
            if let Some(offset) = offset {
                push_offset(&offset.value, out);
            }
            if let Some(limit) = limit {
                expr_offsets(limit, out);
            }
            for expr in limit_by {
                expr_offsets(expr, out);
            }
        }
        Some(LimitClause::OffsetCommaLimit { offset, limit }) => {
            push_offset(offset, out);
            expr_offsets(limit, out);
        }
        None => {}
    }
}

fn push_offset(expr: &Expr, out: &mut Vec<OffsetUse>) {
    out.push(if is_zero(expr) {
        OffsetUse::Zero
    } else {
        OffsetUse::Other
    });
    expr_offsets(expr, out);
}

fn is_zero(expr: &Expr) -> bool {
    matches!(
        nested(expr),
        Expr::Value(value) if matches!(&value.value, Value::Number(text, _) if text == "0")
    )
}

fn set_expr_offsets(expr: &SetExpr, out: &mut Vec<OffsetUse>) {
    match expr {
        SetExpr::Select(select) => select_offsets(select, out),
        SetExpr::Query(query) => query_offsets(query, out),
        SetExpr::SetOperation { left, right, .. } => {
            set_expr_offsets(left, out);
            set_expr_offsets(right, out);
        }
        SetExpr::Values(Values { rows, .. }) => {
            for row in rows {
                for expr in row.iter() {
                    expr_offsets(expr, out);
                }
            }
        }
        SetExpr::Insert(stmt) | SetExpr::Update(stmt) | SetExpr::Delete(stmt) => {
            super::statement_offsets(stmt, out);
        }
        _ => {}
    }
}

fn select_offsets(select: &Select, out: &mut Vec<OffsetUse>) {
    for item in &select.projection {
        select_item_offsets(item, out);
    }
    if let Some(selection) = &select.selection {
        expr_offsets(selection, out);
    }
    if let Some(having) = &select.having {
        expr_offsets(having, out);
    }
    for table in &select.from {
        table_with_joins_offsets(table, out);
    }
    if let GroupByExpr::Expressions(exprs, _) = &select.group_by {
        for expr in exprs {
            expr_offsets(expr, out);
        }
    }
    if let Some(Distinct::On(exprs)) = select.distinct.as_ref() {
        for expr in exprs {
            expr_offsets(expr, out);
        }
    }
    for window in &select.named_window {
        named_window_offsets(window, out);
    }
}

pub(super) fn select_item_offsets(item: &SelectItem, out: &mut Vec<OffsetUse>) {
    match item {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
            expr_offsets(expr, out);
        }
        _ => {}
    }
}

fn named_window_offsets(
    NamedWindowDefinition(_, expr): &NamedWindowDefinition,
    out: &mut Vec<OffsetUse>,
) {
    if let NamedWindowExpr::WindowSpec(WindowSpec {
        partition_by,
        order_by,
        ..
    }) = expr
    {
        for expr in partition_by {
            expr_offsets(expr, out);
        }
        for item in order_by {
            expr_offsets(&item.expr, out);
        }
    }
}

pub(super) fn expr_offsets(expr: &sqlparser::ast::Expr, out: &mut Vec<OffsetUse>) {
    walk_expr(expr, out);
}

pub(super) fn table_with_joins_offsets(
    table: &sqlparser::ast::TableWithJoins,
    out: &mut Vec<OffsetUse>,
) {
    expr::table_with_joins_offsets(table, out);
}

fn nested(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => nested(inner),
        other => other,
    }
}
