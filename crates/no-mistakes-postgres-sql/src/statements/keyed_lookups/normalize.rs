use super::expr_norm::{canonical_type, normalize_expr, normalize_function, normalize_order_by};
use super::scope::ScopeSnapshot;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use sqlparser::ast::{BinaryOperator, Expr, OrderByExpr, Value};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct LookupKey {
    pub(super) relation: String,
    pub(super) equalities: Vec<(String, String)>,
    pub(super) residuals: Vec<String>,
    pub(super) order_by: Vec<String>,
    pub(super) limit: Option<String>,
}

pub(super) fn extract_lookup_key(
    relation: &str,
    self_qualifier: &str,
    single_table_subquery: bool,
    predicate: &Expr,
    outer: &ScopeSnapshot,
    order_by: &[OrderByExpr],
    limit: Option<String>,
) -> Option<(LookupKey, String)> {
    let mut conjuncts = Vec::new();
    flatten_and(predicate, &mut conjuncts);
    let mut equalities = Vec::new();
    let mut display_pairs = Vec::new();
    let mut residuals = Vec::new();
    let mut has_key_anchor = false;
    for conjunct in conjuncts {
        if let Some((col, norm_rhs, anchor, display_rhs)) =
            try_key_equality(conjunct, self_qualifier, single_table_subquery, outer)
        {
            has_key_anchor |= anchor;
            equalities.push((col.clone(), norm_rhs));
            display_pairs.push((col, display_rhs));
        } else {
            let residual =
                normalize_expr(conjunct, Some(self_qualifier), single_table_subquery, outer)?;
            residuals.push(residual);
        }
    }
    if equalities.is_empty() || !has_key_anchor {
        return None;
    }
    let norm_order = normalize_order_by(order_by, self_qualifier, single_table_subquery, outer)?;
    equalities.sort();
    equalities.dedup();
    residuals.sort();
    residuals.dedup();
    let key_display = display_pairs
        .into_iter()
        .map(|(col, rhs)| format!("{col} = {rhs}"))
        .collect::<Vec<_>>()
        .join(", ");
    Some((
        LookupKey {
            relation: relation.to_owned(),
            equalities,
            residuals,
            order_by: norm_order,
            limit,
        },
        key_display,
    ))
}

fn flatten_and<'a>(expr: &'a Expr, out: &mut Vec<&'a Expr>) {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            flatten_and(left, out);
            flatten_and(right, out);
        }
        other => out.push(other),
    }
}

fn try_key_equality(
    expr: &Expr,
    self_qualifier: &str,
    single_table_subquery: bool,
    outer: &ScopeSnapshot,
) -> Option<(String, String, bool, String)> {
    let Expr::BinaryOp {
        left,
        op: BinaryOperator::Eq,
        right,
    } = unwrap_expr(expr)
    else {
        return None;
    };
    try_oriented(left, right, self_qualifier, single_table_subquery, outer)
        .or_else(|| try_oriented(right, left, self_qualifier, single_table_subquery, outer))
}

fn try_oriented(
    self_side: &Expr,
    outer_side: &Expr,
    self_qualifier: &str,
    single_table_subquery: bool,
    outer: &ScopeSnapshot,
) -> Option<(String, String, bool, String)> {
    let (self_col, was_unqualified) =
        self_column(self_side, self_qualifier, single_table_subquery)?;
    let (norm_rhs, anchor) = normalize_outer(outer_side, self_qualifier, outer)?;
    if was_unqualified && !anchor {
        return None;
    }
    Some((
        self_col,
        norm_rhs,
        anchor,
        unwrap_expr(outer_side).to_string(),
    ))
}

fn self_column(
    expr: &Expr,
    self_qualifier: &str,
    single_table_subquery: bool,
) -> Option<(String, bool)> {
    match unwrap_expr(expr) {
        Expr::CompoundIdentifier(parts) if parts.len() == 2 => {
            (ident_key(&parts[0]) == self_qualifier).then(|| (ident_key(&parts[1]), false))
        }
        Expr::Identifier(ident) if single_table_subquery => Some((ident_key(ident), true)),
        _ => None,
    }
}

fn normalize_outer(
    expr: &Expr,
    self_qualifier: &str,
    outer: &ScopeSnapshot,
) -> Option<(String, bool)> {
    let mut anchor = false;
    let text = normalize_outer_inner(expr, self_qualifier, outer, &mut anchor)?;
    Some((text, anchor))
}

fn normalize_outer_inner(
    expr: &Expr,
    self_qualifier: &str,
    outer: &ScopeSnapshot,
    anchor: &mut bool,
) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::CompoundIdentifier(parts) if parts.len() == 2 => {
            let qual = ident_key(&parts[0]);
            if qual == self_qualifier {
                return None;
            }
            let binding = outer.resolve_qualifier(&qual)?;
            *anchor = true;
            Some(format!("b{binding}.{}", ident_key(&parts[1])))
        }
        Expr::Identifier(_) => None,
        Expr::Value(value) => {
            if matches!(value.value, Value::Placeholder(_)) {
                *anchor = true;
            }
            Some(format!("val:{value}"))
        }
        Expr::Cast {
            expr, data_type, ..
        } => {
            let inner = normalize_outer_inner(expr, self_qualifier, outer, anchor)?;
            Some(format!("cast({inner},{})", canonical_type(data_type)))
        }
        Expr::TypedString(typed) => {
            *anchor = true;
            Some(format!(
                "typed({},{})",
                canonical_type(&typed.data_type),
                typed.value
            ))
        }
        Expr::UnaryOp { op, expr } => {
            let inner = normalize_outer_inner(expr, self_qualifier, outer, anchor)?;
            Some(format!("unary({op},{inner})"))
        }
        Expr::BinaryOp { left, op, right } => {
            let l = normalize_outer_inner(left, self_qualifier, outer, anchor)?;
            let r = normalize_outer_inner(right, self_qualifier, outer, anchor)?;
            Some(format!("bin({op},{l},{r})"))
        }
        Expr::Function(function) => normalize_function(function, |arg| {
            normalize_outer_inner(arg, self_qualifier, outer, anchor)
        }),
        _ => None,
    }
}
