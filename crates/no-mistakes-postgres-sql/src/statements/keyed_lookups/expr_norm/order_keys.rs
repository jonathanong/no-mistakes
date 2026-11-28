use super::ExprNormalizer;
use crate::idents::unwrap_expr;
use crate::order::canonical_order_options;
use sqlparser::ast::{Expr, OrderByExpr, Value};

impl ExprNormalizer<'_> {
    pub(in crate::statements::keyed_lookups) fn order_by(
        &mut self,
        order_by: &[OrderByExpr],
    ) -> Option<Vec<String>> {
        order_by.iter().map(|item| self.order_key(item)).collect()
    }

    fn order_key(&mut self, item: &OrderByExpr) -> Option<String> {
        let expr = unwrap_expr(&item.expr);
        // PostgreSQL can resolve a bare name or position against the SELECT list.
        if matches!(expr, Expr::Identifier(_))
            || matches!(expr, Expr::Value(value) if matches!(value.value, Value::Number(..)))
        {
            return None;
        }
        let expr = self.normalize(expr)?;
        let (ascending, nulls_first) = canonical_order_options(&item.options)?;
        Some(format!("{expr}:{ascending}:{nulls_first}"))
    }
}
