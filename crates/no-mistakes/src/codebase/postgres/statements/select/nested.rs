use super::super::SqlSelectFact;
use sqlparser::ast::{Expr, GroupByExpr, Select, SelectItem};

pub(super) fn collect(
    sql: &str,
    select: &Select,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    walk_optional(sql, select.selection.as_ref(), ctes, in_insert_select, out);
    walk_optional(sql, select.having.as_ref(), ctes, in_insert_select, out);
    for item in &select.projection {
        if let SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } = item {
            walk_expr(sql, expr, ctes, in_insert_select, out);
        }
    }
    if let GroupByExpr::Expressions(exprs, _) = &select.group_by {
        for expr in exprs {
            walk_expr(sql, expr, ctes, in_insert_select, out);
        }
    }
    for table in &select.from {
        for join in &table.joins {
            if let Some(expr) = super::join_expr(&join.join_operator) {
                walk_expr(sql, expr, ctes, in_insert_select, out);
            }
        }
    }
}

pub(super) fn walk_expr(
    sql: &str,
    expr: &Expr,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    use sqlparser::ast::Visit;
    let _ = expr.visit(&mut Queries {
        sql,
        ctes,
        in_insert_select,
        out,
        depth: 0,
        in_exists: false,
    });
}

struct Queries<'a> {
    sql: &'a str,
    ctes: &'a [String],
    in_insert_select: bool,
    out: &'a mut Vec<SqlSelectFact>,
    depth: usize,
    in_exists: bool,
}

impl sqlparser::ast::Visitor for Queries<'_> {
    type Break = ();
    fn pre_visit_expr(&mut self, expr: &Expr) -> std::ops::ControlFlow<()> {
        if self.depth == 0 {
            self.in_exists = matches!(expr, Expr::Exists { .. });
        }
        std::ops::ControlFlow::Continue(())
    }
    fn pre_visit_query(&mut self, query: &sqlparser::ast::Query) -> std::ops::ControlFlow<()> {
        if self.depth == 0 {
            super::collect_query(
                self.sql,
                query,
                self.ctes,
                self.in_insert_select,
                self.in_exists,
                self.out,
            );
        }
        self.depth += 1;
        std::ops::ControlFlow::Continue(())
    }
    fn post_visit_query(&mut self, _: &sqlparser::ast::Query) -> std::ops::ControlFlow<()> {
        self.depth -= 1;
        std::ops::ControlFlow::Continue(())
    }
}

pub(super) fn collect_query_expressions(
    sql: &str,
    query: &sqlparser::ast::Query,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    use sqlparser::ast::Visit;
    let mut visitor = Queries {
        sql,
        ctes,
        in_insert_select,
        out,
        depth: 0,
        in_exists: false,
    };
    let _ = query.order_by.visit(&mut visitor);
    let _ = query.limit_clause.visit(&mut visitor);
    let _ = query.fetch.visit(&mut visitor);
}

fn walk_optional(
    sql: &str,
    expr: Option<&Expr>,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    if let Some(expr) = expr {
        walk_expr(sql, expr, ctes, in_insert_select, out);
    }
}
