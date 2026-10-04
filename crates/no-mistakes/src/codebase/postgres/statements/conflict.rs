use super::{SqlConflictArbiter, SqlConflictWhereProof};
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{BinaryOperator, ConflictTarget, Expr, UnaryOperator};

pub(super) fn arbiter(target: &Option<ConflictTarget>) -> SqlConflictArbiter {
    match target {
        Some(ConflictTarget::Columns(columns)) => {
            SqlConflictArbiter::Columns(columns.iter().map(|ident| ident.value.clone()).collect())
        }
        Some(ConflictTarget::OnConstraint(name)) => {
            SqlConflictArbiter::Constraint(relation_name(name))
        }
        None => SqlConflictArbiter::Unknown,
    }
}

pub(super) fn where_proof_at(
    selection: Option<&Expr>,
    positions: super::value::PlaceholderPositions<'_>,
) -> SqlConflictWhereProof {
    let mut proof = SqlConflictWhereProof::default();
    if let Some(expr) = selection {
        collect_proof(expr, &mut proof, false, positions);
    }
    proof
}

fn collect_proof(
    expr: &Expr,
    proof: &mut SqlConflictWhereProof,
    under_or: bool,
    positions: super::value::PlaceholderPositions<'_>,
) {
    match expr {
        Expr::Nested(inner) => collect_proof(inner, proof, under_or, positions),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            if !under_or {
                classify_leaf(expr, proof, positions);
            }
            collect_proof(left, proof, under_or, positions);
            collect_proof(right, proof, under_or, positions);
        }
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Or,
            right,
        } => {
            proof.disjunctive = true;
            collect_proof(left, proof, true, positions);
            collect_proof(right, proof, true, positions);
        }
        other if !under_or => classify_leaf(other, proof, positions),
        _ => {}
    }
}

fn classify_leaf(
    expr: &Expr,
    proof: &mut SqlConflictWhereProof,
    positions: super::value::PlaceholderPositions<'_>,
) {
    if let Some(column) = distinct_from_excluded(expr, positions) {
        proof.distinct_from_excluded.push(column);
        return;
    }
    if let Some(column) = null_and_excluded_not_null(expr, positions) {
        proof.null_and_excluded_not_null.push(column);
    }
}

fn distinct_from_excluded(
    expr: &Expr,
    positions: super::value::PlaceholderPositions<'_>,
) -> Option<String> {
    let Expr::IsDistinctFrom(left, right) = expr else {
        return None;
    };
    excluded_pair(left, right, positions).or_else(|| excluded_pair(right, left, positions))
}

fn null_and_excluded_not_null(
    expr: &Expr,
    positions: super::value::PlaceholderPositions<'_>,
) -> Option<String> {
    let Expr::BinaryOp {
        left,
        op: BinaryOperator::And,
        right,
    } = expr
    else {
        return None;
    };
    paired_null_excluded(left, right, positions)
        .or_else(|| paired_null_excluded(right, left, positions))
}

fn paired_null_excluded(
    null_side: &Expr,
    excluded_side: &Expr,
    positions: super::value::PlaceholderPositions<'_>,
) -> Option<String> {
    let column = is_null(null_side, positions)?;
    let excluded = is_not_null(excluded_side, positions)?;
    column.eq_ignore_ascii_case(&excluded).then_some(column)
}

fn is_null(expr: &Expr, positions: super::value::PlaceholderPositions<'_>) -> Option<String> {
    match expr {
        Expr::IsNull(inner) => super::value::self_ref_column_at(inner, positions),
        _ => None,
    }
}

fn is_not_null(expr: &Expr, positions: super::value::PlaceholderPositions<'_>) -> Option<String> {
    match expr {
        Expr::IsNotNull(inner) => super::value::excluded_column_at(inner, positions),
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr,
        } => match expr.as_ref() {
            Expr::IsNull(inner) => super::value::excluded_column_at(inner, positions),
            _ => None,
        },
        _ => None,
    }
}

fn excluded_pair(
    left: &Expr,
    right: &Expr,
    positions: super::value::PlaceholderPositions<'_>,
) -> Option<String> {
    let self_col = super::value::self_ref_column_at(left, positions)?;
    let excluded = super::value::excluded_column_at(right, positions)?;
    self_col.eq_ignore_ascii_case(&excluded).then_some(self_col)
}
