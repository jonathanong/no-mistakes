use super::scope::{self, qualifier};
use super::SqlTransformedColumnUseFact;
use sqlparser::ast::{BinaryOperator, Expr, JoinOperator, TableWithJoins, Value};
use std::collections::BTreeSet;
use std::ops::ControlFlow;

pub(super) fn mark(
    tables: &[TableWithJoins],
    selection: &Expr,
    ctes: &[String],
    uses: &mut [SqlTransformedColumnUseFact],
) {
    let relations = scope::base_relations(tables, ctes);
    let mut nullable = BTreeSet::new();
    for table in tables {
        let mut preceding = Vec::new();
        scope::push_factor(&table.relation, ctes, &mut preceding);
        for join in &table.joins {
            let mut right = Vec::new();
            scope::push_factor(&join.relation, ctes, &mut right);
            match join.join_operator {
                JoinOperator::Left(_) | JoinOperator::LeftOuter(_) => {
                    nullable.extend(right.iter().map(identity));
                }
                JoinOperator::Right(_) | JoinOperator::RightOuter(_) => {
                    nullable.extend(preceding.iter().map(identity));
                }
                JoinOperator::FullOuter(_) => {
                    nullable.extend(preceding.iter().chain(&right).map(identity));
                }
                _ => {}
            }
            preceding.extend(right);
        }
    }
    scope::visit_expressions(selection, &mut |expr| {
        let Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } = expr
        else {
            return;
        };
        for (value, expected) in [(left, right), (right, left)] {
            if !matches!(unnest(expected), Expr::Value(value) if matches!(&value.value, Value::Number(number, _) if number == "1"))
            {
                continue;
            }
            let Expr::Function(function) = unnest(value) else {
                continue;
            };
            let name = crate::codebase::postgres::idents::object_name_identity(&function.name);
            if !matches!(name.as_str(), "num_nonnulls" | "pg_catalog.num_nonnulls") {
                continue;
            }
            let mut owners = BTreeSet::new();
            let proof = sqlparser::ast::visit_expressions(value, |expr| {
                if matches!(
                    expr,
                    Expr::Subquery(_) | Expr::Exists { .. } | Expr::InSubquery { .. }
                ) {
                    return ControlFlow::Break(());
                }
                if matches!(expr, Expr::Identifier(_) | Expr::CompoundIdentifier(_)) {
                    let Some(qualifier) = qualifier(expr) else {
                        return ControlFlow::Break(());
                    };
                    if scope::owner(&relations, Some(&qualifier)).0.is_empty() {
                        return ControlFlow::Break(());
                    }
                    owners.insert(qualifier);
                }
                ControlFlow::Continue(())
            });
            if proof.is_continue() && owners.len() > 1 && !owners.is_disjoint(&nullable) {
                use sqlparser::ast::Spanned;
                let at = function.name.span().start;
                for use_ in uses
                    .iter_mut()
                    .filter(|use_| use_.expression_site == (at.line as usize, at.column as usize))
                {
                    use_.outer_join_residual = true;
                }
            }
        }
    });
}

fn identity(relation: &super::super::select::uses::scope::BaseRel) -> String {
    relation
        .alias
        .clone()
        .unwrap_or_else(|| relation.table.clone())
}
fn unnest(mut expression: &Expr) -> &Expr {
    while let Expr::Nested(inner) = expression {
        expression = inner;
    }
    expression
}
