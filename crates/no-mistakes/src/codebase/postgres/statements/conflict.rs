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

pub(super) fn where_proof(selection: Option<&Expr>) -> SqlConflictWhereProof {
    let mut proof = SqlConflictWhereProof::default();
    if let Some(expr) = selection {
        collect_proof(expr, &mut proof, false);
    }
    proof
}

fn collect_proof(expr: &Expr, proof: &mut SqlConflictWhereProof, under_or: bool) {
    match expr {
        Expr::Nested(inner) => collect_proof(inner, proof, under_or),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            if !under_or {
                classify_leaf(expr, proof);
            }
            collect_proof(left, proof, under_or);
            collect_proof(right, proof, under_or);
        }
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Or,
            right,
        } => {
            proof.disjunctive = true;
            collect_proof(left, proof, true);
            collect_proof(right, proof, true);
        }
        other if !under_or => classify_leaf(other, proof),
        _ => {}
    }
}

fn classify_leaf(expr: &Expr, proof: &mut SqlConflictWhereProof) {
    if let Some(column) = distinct_from_excluded(expr) {
        proof.distinct_from_excluded.push(column);
        return;
    }
    if let Some(column) = null_and_excluded_not_null(expr) {
        proof.null_and_excluded_not_null.push(column);
    }
}

fn distinct_from_excluded(expr: &Expr) -> Option<String> {
    let Expr::IsDistinctFrom(left, right) = expr else {
        return None;
    };
    excluded_pair(left, right).or_else(|| excluded_pair(right, left))
}

fn null_and_excluded_not_null(expr: &Expr) -> Option<String> {
    let Expr::BinaryOp {
        left,
        op: BinaryOperator::And,
        right,
    } = expr
    else {
        return None;
    };
    paired_null_excluded(left, right).or_else(|| paired_null_excluded(right, left))
}

fn paired_null_excluded(null_side: &Expr, excluded_side: &Expr) -> Option<String> {
    let column = is_null(null_side)?;
    let excluded = is_not_null(excluded_side)?;
    column.eq_ignore_ascii_case(&excluded).then_some(column)
}

fn is_null(expr: &Expr) -> Option<String> {
    match expr {
        Expr::IsNull(inner) => super::value::self_ref_column(inner),
        _ => None,
    }
}

fn is_not_null(expr: &Expr) -> Option<String> {
    match expr {
        Expr::IsNotNull(inner) => super::value::excluded_column(inner),
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr,
        } => match expr.as_ref() {
            Expr::IsNull(inner) => super::value::excluded_column(inner),
            _ => None,
        },
        _ => None,
    }
}

fn excluded_pair(left: &Expr, right: &Expr) -> Option<String> {
    let self_col = super::value::self_ref_column(left)?;
    let excluded = super::value::excluded_column(right)?;
    self_col.eq_ignore_ascii_case(&excluded).then_some(self_col)
}
