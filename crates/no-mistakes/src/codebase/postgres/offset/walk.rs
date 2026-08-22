use sqlparser::ast::{
    Distinct, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, GroupByExpr,
    JoinConstraint, JoinOperator, LimitClause, NamedWindowDefinition, NamedWindowExpr, Query,
    Select, SelectItem, SetExpr, TableFactor, TableFunctionArgs, TableWithJoins, Values,
    WindowSpec,
};

pub(super) fn query_has_offset(query: &Query) -> bool {
    if limit_clause_has_offset(query.limit_clause.as_ref()) {
        return true;
    }
    if query.with.as_ref().is_some_and(|with| {
        with.cte_tables
            .iter()
            .any(|cte| query_has_offset(&cte.query))
    }) {
        return true;
    }
    if let Some(order_by) = &query.order_by {
        if let sqlparser::ast::OrderByKind::Expressions(exprs) = &order_by.kind {
            if exprs.iter().any(|item| expr_has_offset_query(&item.expr)) {
                return true;
            }
        }
    }
    set_expr_has_offset(&query.body)
}

fn limit_clause_has_offset(clause: Option<&LimitClause>) -> bool {
    match clause {
        Some(LimitClause::LimitOffset {
            limit,
            offset,
            limit_by,
        }) => {
            offset.is_some()
                || limit.as_ref().is_some_and(expr_has_offset_query)
                || limit_by.iter().any(expr_has_offset_query)
        }
        other => other.is_some(),
    }
}

fn set_expr_has_offset(expr: &SetExpr) -> bool {
    match expr {
        SetExpr::Select(select) => select_has_offset(select),
        SetExpr::Query(query) => query_has_offset(query),
        SetExpr::SetOperation { left, right, .. } => {
            set_expr_has_offset(left) || set_expr_has_offset(right)
        }
        SetExpr::Values(Values { rows, .. }) => {
            rows.iter().any(|row| row.iter().any(expr_has_offset_query))
        }
        SetExpr::Insert(stmt) | SetExpr::Update(stmt) | SetExpr::Delete(stmt) => {
            super::statement_has_offset(stmt)
        }
        _ => false,
    }
}

fn select_has_offset(select: &Select) -> bool {
    select.projection.iter().any(select_item_has_offset)
        || select.selection.as_ref().is_some_and(expr_has_offset_query)
        || select.having.as_ref().is_some_and(expr_has_offset_query)
        || select.from.iter().any(table_with_joins_has_offset)
        || matches!(
            &select.group_by,
            GroupByExpr::Expressions(exprs, _) if exprs.iter().any(expr_has_offset_query)
        )
        || matches!(
            select.distinct.as_ref(),
            Some(Distinct::On(exprs)) if exprs.iter().any(expr_has_offset_query)
        )
        || select.named_window.iter().any(named_window_has_offset)
}

pub(super) fn select_item_has_offset(item: &SelectItem) -> bool {
    match item {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
            expr_has_offset_query(expr)
        }
        _ => false,
    }
}

fn named_window_has_offset(NamedWindowDefinition(_, expr): &NamedWindowDefinition) -> bool {
    matches!(
        expr,
        NamedWindowExpr::WindowSpec(WindowSpec {
            partition_by,
            order_by,
            ..
        }) if partition_by.iter().any(expr_has_offset_query)
            || order_by.iter().any(|item| expr_has_offset_query(&item.expr))
    )
}

pub(super) fn table_with_joins_has_offset(table: &TableWithJoins) -> bool {
    table_factor_has_offset(&table.relation)
        || table.joins.iter().any(|join| {
            table_factor_has_offset(&join.relation) || join_operator_has_offset(&join.join_operator)
        })
}

fn table_factor_has_offset(factor: &TableFactor) -> bool {
    match factor {
        TableFactor::Derived { subquery, .. } => query_has_offset(subquery),
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => table_with_joins_has_offset(table_with_joins),
        TableFactor::Table {
            args: Some(TableFunctionArgs { args, .. }),
            ..
        } => args.iter().any(|arg| match arg {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => expr_has_offset_query(expr),
            _ => false,
        }),
        _ => false,
    }
}

fn join_operator_has_offset(op: &JoinOperator) -> bool {
    match op {
        JoinOperator::Join(c)
        | JoinOperator::Inner(c)
        | JoinOperator::Left(c)
        | JoinOperator::LeftOuter(c)
        | JoinOperator::Right(c)
        | JoinOperator::RightOuter(c)
        | JoinOperator::FullOuter(c) => {
            matches!(c, JoinConstraint::On(expr) if expr_has_offset_query(expr))
        }
        _ => false,
    }
}

pub(super) fn expr_has_offset_query(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Subquery(query)
        | Expr::Exists {
            subquery: query, ..
        }
        | Expr::InSubquery {
            subquery: query, ..
        } => query_has_offset(query),
        Expr::BinaryOp { left, right, .. } => {
            expr_has_offset_query(left) || expr_has_offset_query(right)
        }
        Expr::UnaryOp { expr, .. } => expr_has_offset_query(expr),
        Expr::Function(Function { args, .. }) => function_args_has_offset(args),
        _ => false,
    }
}

fn function_args_has_offset(args: &FunctionArguments) -> bool {
    match args {
        FunctionArguments::List(list) => list.args.iter().any(|arg| match arg {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => expr_has_offset_query(expr),
            _ => false,
        }),
        _ => false,
    }
}

fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => unwrap_expr(inner),
        other => other,
    }
}
