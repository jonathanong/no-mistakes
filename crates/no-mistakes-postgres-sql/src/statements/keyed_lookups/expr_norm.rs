pub(super) mod function;

use super::scope::ScopeSnapshot;
use crate::idents::{ident_key, object_name_identity, unwrap_expr};
use sqlparser::ast::{ArrayElemTypeDef, DataType, Expr, Ident, InTarget, OrderByExpr, Value};

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
                    canonical_type(&typed.data_type),
                    typed.value
                ))
            }
            Expr::Cast {
                expr, data_type, ..
            } => {
                let inner = self.normalize(expr)?;
                Some(format!("cast({inner},{})", canonical_type(data_type)))
            }
            Expr::UnaryOp { op, expr } => {
                let inner = self.normalize(expr)?;
                Some(format!("unary({op},{inner})"))
            }
            Expr::BinaryOp { left, op, right } => Some(format!(
                "bin({op},{},{})",
                self.normalize(left)?,
                self.normalize(right)?
            )),
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
        let binding = self.outer.resolve_qualifier(&qualifier)?;
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
            Expr::IsNull(inner) => ("is_null", inner),
            Expr::IsNotNull(inner) => ("is_not_null", inner),
            Expr::IsTrue(inner) => ("is_true", inner),
            Expr::IsFalse(inner) => ("is_false", inner),
            Expr::IsDistinctFrom(left, right) | Expr::IsNotDistinctFrom(left, right) => {
                let operator = if matches!(expr, Expr::IsDistinctFrom(..)) {
                    "is_distinct"
                } else {
                    "is_not_distinct"
                };
                return Some(format!(
                    "{operator}({},{})",
                    self.normalize(left)?,
                    self.normalize(right)?
                ));
            }
            Expr::In {
                expr,
                target: InTarget::List(items),
                negated,
                ..
            } => return self.in_list(expr, items, *negated),
            _ => return None,
        };
        Some(format!("{operator}({})", self.normalize(inner)?))
    }

    fn in_list(&mut self, expr: &Expr, items: &[Expr], negated: bool) -> Option<String> {
        let target = self.normalize(expr)?;
        let mut items = items
            .iter()
            .map(|item| self.normalize(item))
            .collect::<Option<Vec<_>>>()?;
        items.sort();
        Some(format!("in({negated},{target},[{}])", items.join(",")))
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
        Some(format!(
            "{expr}:{:?}:{:?}",
            item.options.sort, item.options.nulls_first
        ))
    }
}

fn canonical_type(data_type: &DataType) -> String {
    match data_type {
        DataType::Custom(name, mods) => {
            let base = object_name_identity(name);
            if mods.is_empty() {
                base
            } else {
                let mods = mods
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{base}({mods})")
            }
        }
        DataType::Array(
            ArrayElemTypeDef::AngleBracket(inner)
            | ArrayElemTypeDef::SquareBracket(inner, _)
            | ArrayElemTypeDef::Parenthesis(inner)
            | ArrayElemTypeDef::Qualified(inner, _),
        ) => format!("{}[]", canonical_type(inner)),
        other => other.to_string().to_ascii_lowercase(),
    }
}
