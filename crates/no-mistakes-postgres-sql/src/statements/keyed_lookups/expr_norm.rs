pub(super) mod function;
mod operators;
pub(super) mod types;

use super::scope::ScopeSnapshot;
use crate::idents::{ident_key, unwrap_expr};
use crate::order::canonical_order_options;
use sqlparser::ast::{CastKind, Expr, Ident, InTarget, OrderByExpr, Value};

pub(super) struct ExprNormalizer<'a> {
    self_qualifier: Option<&'a str>,
    allow_unqualified_self: bool,
    outer: &'a ScopeSnapshot,
    key_anchor: Option<bool>,
}

impl<'a> ExprNormalizer<'a> {
    pub(super) fn new(
        self_qualifier: Option<&'a str>,
        allow_unqualified_self: bool,
        outer: &'a ScopeSnapshot,
    ) -> Self {
        Self {
            self_qualifier,
            allow_unqualified_self,
            outer,
            key_anchor: None,
        }
    }

    pub(super) fn outer_key(
        expr: &Expr,
        self_qualifier: &'a str,
        outer: &'a ScopeSnapshot,
    ) -> Option<(String, bool)> {
        let mut normalizer = Self {
            key_anchor: Some(false),
            ..Self::new(Some(self_qualifier), false, outer)
        };
        let text = normalizer.normalize(expr)?;
        Some((text, normalizer.key_anchor == Some(true)))
    }

    pub(super) fn normalize(&mut self, expr: &Expr) -> Option<String> {
        match unwrap_expr(expr) {
            Expr::CompoundIdentifier(parts) if parts.len() == 2 => {
                self.column(&parts[0], &parts[1])
            }
            Expr::Identifier(ident) if self.allow_unqualified_self => {
                Some(format!("self.{}", ident_key(ident)))
            }
            Expr::Value(value) => {
                if matches!(value.value, Value::Placeholder(_)) {
                    self.mark_anchor();
                }
                Some(format!("val:{value}"))
            }
            Expr::TypedString(typed) => {
                self.mark_anchor();
                Some(format!(
                    "typed({},{})",
                    atom(&types::cast_type(&typed.data_type)?),
                    atom(&typed.value.to_string())
                ))
            }
            Expr::Cast {
                kind: CastKind::Cast | CastKind::DoubleColon,
                expr,
                data_type,
                format: None,
                ..
            } if types::has_builtin_type(expr) => {
                let inner = self.normalize(expr)?;
                Some(format!(
                    "cast({},{})",
                    atom(&inner),
                    atom(&types::cast_type(data_type)?)
                ))
            }
            Expr::IsNull(inner) => Some(format!("is_null({})", self.normalize(inner)?)),
            Expr::IsNotNull(inner) => Some(format!("is_not_null({})", self.normalize(inner)?)),
            Expr::UnaryOp { op, expr } if operators::unary_is_proven(*op, expr) => {
                let inner = self.normalize(expr)?;
                Some(format!("unary({op},{})", atom(&inner)))
            }
            Expr::BinaryOp { left, op, right } if operators::binary_is_proven(op, left, right) => {
                Some(format!(
                    "bin({op},{},{})",
                    atom(&self.normalize(left)?),
                    atom(&self.normalize(right)?)
                ))
            }
            Expr::Function(function) => function::normalize(function, |arg| self.normalize(arg)),
            predicate if self.key_anchor.is_none() => self.predicate(predicate),
            _ => None,
        }
    }

    fn column(&mut self, qualifier: &Ident, column: &Ident) -> Option<String> {
        let qualifier = ident_key(qualifier);
        let column = ident_key(column);
        if self.self_qualifier == Some(qualifier.as_str()) {
            return self.key_anchor.is_none().then(|| format!("self.{column}"));
        }
        let binding = self.outer.resolve_column(&qualifier, &column)?;
        self.mark_anchor();
        Some(format!("b{binding}.{column}"))
    }

    fn mark_anchor(&mut self) {
        if let Some(anchor) = &mut self.key_anchor {
            *anchor = true;
        }
    }

    fn predicate(&mut self, expr: &Expr) -> Option<String> {
        let (operator, inner) = match expr {
            Expr::IsTrue(inner) => ("is_true", inner),
            Expr::IsFalse(inner) => ("is_false", inner),
            Expr::IsDistinctFrom(left, right) | Expr::IsNotDistinctFrom(left, right)
                if operators::comparable(left, right) =>
            {
                let operator = if matches!(expr, Expr::IsDistinctFrom(..)) {
                    "is_distinct"
                } else {
                    "is_not_distinct"
                };
                return Some(format!(
                    "{operator}({},{})",
                    atom(&self.normalize(left)?),
                    atom(&self.normalize(right)?)
                ));
            }
            Expr::In {
                expr,
                target: InTarget::List(items),
                negated,
                ..
            } if operators::list_operands_are_proven(expr, items) => {
                return self.in_list(expr, items, *negated)
            }
            _ => return None,
        };
        if operators::is_proven_boolean(inner) {
            Some(format!("{operator}({})", self.normalize(inner)?))
        } else {
            None
        }
    }

    fn in_list(&mut self, expr: &Expr, items: &[Expr], negated: bool) -> Option<String> {
        let target = self.normalize(expr)?;
        let mut items = items
            .iter()
            .map(|item| self.normalize(item))
            .collect::<Option<Vec<_>>>()?;
        items.sort();
        Some(format!(
            "in({negated},{}[{}])",
            atom(&target),
            argument_list(&items)
        ))
    }

    pub(super) fn order_by(&mut self, order_by: &[OrderByExpr]) -> Option<Vec<String>> {
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

fn atom(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn argument_list(items: &[String]) -> String {
    items.iter().map(|item| atom(item)).collect()
}
