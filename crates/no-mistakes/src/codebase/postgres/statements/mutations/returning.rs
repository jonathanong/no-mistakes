use super::SqlSelectFact;

pub(super) fn walk_returning(
    sql: &str,
    returning: Option<&[sqlparser::ast::SelectItem]>,
    ctes: &[String],
    selects: &mut Vec<SqlSelectFact>,
) {
    for item in returning.into_iter().flatten() {
        if let sqlparser::ast::SelectItem::UnnamedExpr(expr)
        | sqlparser::ast::SelectItem::ExprWithAlias { expr, .. } = item
        {
            super::super::select::walk_expr(sql, expr, ctes, false, selects);
        }
    }
}
