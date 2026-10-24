use super::{Clause, SqlTransformedColumnUseFact};
use crate::codebase::postgres::idents::{
    ident_key, relation_part_key, relation_suffix_key, visit_child_exprs, visit_function_args,
};
use crate::codebase::postgres::statements::select::uses::{columns::bare, scope::BaseRel};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::{BinaryOperator as Op, Expr, FunctionArguments, Spanned, UnaryOperator};

pub(super) fn collect(
    expression: &Expr,
    clause: Clause,
    relations: &[BaseRel],
    positions: PlaceholderPositions<'_>,
    out: &mut Vec<SqlTransformedColumnUseFact>,
) {
    Inputs {
        clause,
        relations,
        positions,
        out,
    }
    .walk(expression, None);
}
struct Inputs<'a, 'b, 'c> {
    clause: Clause,
    relations: &'a [BaseRel],
    positions: PlaceholderPositions<'b>,
    out: &'c mut Vec<SqlTransformedColumnUseFact>,
}
impl Inputs<'_, '_, '_> {
    fn walk(&mut self, expr: &Expr, transformation: Option<&str>) {
        // A surrounding transformation consumes the boolean result, not its UUID operands.
        let transformation = if boolean_result(expr) {
            None
        } else {
            transformation
        };
        match expr {
            Expr::Case {
                operand,
                conditions,
                else_result,
                ..
            } => {
                if let Some(operand) = operand {
                    self.walk(operand, None);
                }
                for branch in conditions {
                    self.walk(&branch.condition, None);
                    self.walk(&branch.result, transformation);
                }
                if let Some(result) = else_result {
                    self.walk(result, transformation);
                }
            }
            Expr::Function(function) => {
                let name = function.name.to_string();
                let transform = transformation.unwrap_or(&name);
                for args in [&function.parameters, &function.args] {
                    if let FunctionArguments::List(list) = args {
                        // Argument labels and aggregate ordering are syntax, not transformed inputs.
                        visit_function_args(&list.args, &mut |arg| self.walk(arg, Some(transform)));
                    }
                }
                if let Some(filter) = &function.filter {
                    self.walk(filter, None);
                }
            }
            Expr::Cast { expr, .. } => self.walk(expr, Some(transformation.unwrap_or("cast"))),
            Expr::Identifier(_) | Expr::CompoundIdentifier(_) if transformation.is_some() => {
                if let Some((_, column)) = bare(expr, self.positions) {
                    let qualifier = match expr {
                        Expr::CompoundIdentifier(parts) => Some(
                            parts[..parts.len() - 1]
                                .iter()
                                .map(|part| relation_part_key(&ident_key(part)))
                                .collect::<Vec<_>>()
                                .join("."),
                        ),
                        _ => None,
                    };
                    let (table, candidate_tables) = owner(self.relations, qualifier.as_deref());
                    self.out.push(SqlTransformedColumnUseFact {
                        table,
                        column,
                        candidate_tables,
                        clause: self.clause,
                        line: expr.span().start.line.max(1) as usize,
                        column_position: expr.span().start.column.max(1) as usize,
                        transformation: transformation.unwrap().into(),
                    });
                }
            }
            // The shared walker skips scalar subquery columns. Their predicates get their own scope.
            _ => visit_child_exprs(expr, &mut |child| self.walk(child, transformation)),
        }
    }
}
fn owner(relations: &[BaseRel], qualifier: Option<&str>) -> (String, Option<Vec<String>>) {
    let Some(qualifier) = qualifier else {
        return (
            String::new(),
            (!relations.iter().any(|rel| rel.unknown))
                .then(|| relations.iter().map(|rel| rel.table.clone()).collect()),
        );
    };
    let mut matches = relations.iter().filter(|rel| {
        rel.alias.as_deref().map_or_else(
            || rel.table == qualifier || relation_suffix_key(&rel.table) == qualifier,
            |alias| alias == qualifier,
        )
    });
    match (matches.next(), matches.next()) {
        (Some(relation), None) if !relation.unknown => {
            (relation.table.clone(), Some(vec![relation.table.clone()]))
        }
        _ => (String::new(), None),
    }
}

fn boolean_result(expr: &Expr) -> bool {
    match expr {
        Expr::BinaryOp { op, .. } => matches!(
            op,
            Op::Eq
                | Op::NotEq
                | Op::Gt
                | Op::Lt
                | Op::GtEq
                | Op::LtEq
                | Op::Spaceship
                | Op::And
                | Op::Or
                | Op::Xor
                | Op::PGOverlap
                | Op::PGRegexMatch
                | Op::PGRegexIMatch
                | Op::PGRegexNotMatch
                | Op::PGRegexNotIMatch
                | Op::PGLikeMatch
                | Op::PGILikeMatch
                | Op::PGNotLikeMatch
                | Op::PGNotILikeMatch
                | Op::PGStartsWith
                | Op::AtAt
                | Op::AtArrow
                | Op::ArrowAt
                | Op::AtQuestion
                | Op::Question
                | Op::QuestionAnd
                | Op::QuestionPipe
                | Op::Overlaps
        ),
        Expr::UnaryOp { op, .. } => matches!(op, UnaryOperator::Not | UnaryOperator::BangNot),
        _ => matches!(
            expr,
            Expr::IsFalse(_)
                | Expr::IsNotFalse(_)
                | Expr::IsTrue(_)
                | Expr::IsNotTrue(_)
                | Expr::IsNull(_)
                | Expr::IsNotNull(_)
                | Expr::IsUnknown(_)
                | Expr::IsNotUnknown(_)
                | Expr::IsDistinctFrom(..)
                | Expr::IsNotDistinctFrom(..)
                | Expr::IsJson { .. }
                | Expr::IsNormalized { .. }
                | Expr::InList { .. }
                | Expr::InSubquery { .. }
                | Expr::InUnnest { .. }
                | Expr::Between { .. }
                | Expr::Like { .. }
                | Expr::ILike { .. }
                | Expr::SimilarTo { .. }
                | Expr::RLike { .. }
                | Expr::AnyOp { .. }
                | Expr::AllOp { .. }
                | Expr::Exists { .. }
        ),
    }
}
