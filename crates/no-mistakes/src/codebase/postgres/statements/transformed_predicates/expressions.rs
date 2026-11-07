mod columns;
use super::boolean::boolean_result;
use super::scope::{owner, qualifier};
use super::{Clause, SqlTransformedColumnUseFact};
use crate::codebase::postgres::idents::{visit_child_exprs, visit_function_args};
use crate::codebase::postgres::statements::bounds::is_aggregate;
use crate::codebase::postgres::statements::select::uses::{columns::bare, scope::BaseRel};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use crate::codebase::postgres::statements::SqlTransformedColumnProbeFact as Probe;
use sqlparser::ast::{BinaryOperator as Op, Expr, FunctionArguments, Spanned};

pub(super) fn collect(
    expression: &Expr,
    clause: Clause,
    relations: &[BaseRel],
    probed: Option<&[BaseRel]>,
    positions: PlaceholderPositions<'_>,
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    out: &mut Vec<SqlTransformedColumnUseFact>,
) {
    Inputs {
        clause,
        relations,
        positions,
        tokens,
        probed,
        probe_value: None,
        probes: Vec::new(),
        positive_conjunct: true,
        null_test: false,
        out,
    }
    .walk(expression, None);
}
struct Inputs<'a, 'b, 'c> {
    clause: Clause,
    relations: &'a [BaseRel],
    probed: Option<&'a [BaseRel]>,
    probe_value: Option<usize>,
    probes: Vec<Probe>,
    positive_conjunct: bool,
    null_test: bool,
    positions: PlaceholderPositions<'b>,
    tokens: &'a [sqlparser::tokenizer::TokenWithSpan],
    out: &'c mut Vec<SqlTransformedColumnUseFact>,
}
impl Inputs<'_, '_, '_> {
    fn equality_operand(&mut self, operand: &Expr, other: &Expr) {
        let previous = (self.probe_value, self.positive_conjunct);
        self.positive_conjunct = false;
        self.probe_value = bare(other, self.positions).and_then(|(_, column)| {
            let qualifier = qualifier(other);
            let scope = self.probed.unwrap_or(self.relations);
            let (table, candidate_tables) = owner(scope, qualifier.as_deref());
            (self.probed.is_none()
                || (qualifier.is_some()
                    && !table.is_empty()
                    && owner(self.relations, qualifier.as_deref()).0 == table))
                .then(|| {
                    self.probes.push(Probe {
                        table,
                        column,
                        candidate_tables,
                    });
                    self.probes.len() - 1
                })
        });
        self.walk(operand, None);
        (self.probe_value, self.positive_conjunct) = previous;
    }
    fn walk_condition(&mut self, expr: &Expr) {
        let previous = (self.probe_value, self.null_test);
        self.probe_value = None;
        self.null_test = false;
        self.walk(expr, None);
        (self.probe_value, self.null_test) = previous;
    }
    fn walk(&mut self, expr: &Expr, transformation: Option<(&str, (usize, usize))>) {
        let previous = self.probe_value;
        let positive_conjunct = self.positive_conjunct;
        let null_test = self.null_test;
        self.positive_conjunct &= matches!(
            expr,
            Expr::Nested(_)
                | Expr::BinaryOp {
                    op: Op::Eq | Op::And,
                    ..
                }
        );
        if boolean_result(expr) {
            self.null_test = false;
            self.probe_value = None;
        }
        self.walk_value(expr, transformation);
        self.probe_value = previous;
        self.positive_conjunct = positive_conjunct;
        self.null_test = null_test;
    }
    fn walk_value(&mut self, expr: &Expr, transformation: Option<(&str, (usize, usize))>) {
        // A surrounding transformation consumes the boolean result, not its UUID operands.
        let transformation = if boolean_result(expr) {
            None
        } else {
            transformation
        };
        match expr {
            Expr::BinaryOp {
                left,
                op: Op::Eq,
                right,
            } if self.probed.is_some() && self.positive_conjunct => {
                self.equality_operand(left, right);
                self.equality_operand(right, left);
            }
            Expr::IsDistinctFrom(left, right) | Expr::IsNotDistinctFrom(left, right)
                if self.clause == Clause::Where =>
            {
                self.equality_operand(left, right);
                self.equality_operand(right, left);
            }
            Expr::IsNull(value) | Expr::IsNotNull(value) => {
                self.null_test = true;
                self.walk(value, None);
            }
            Expr::Case {
                operand,
                conditions,
                else_result,
                ..
            } => {
                if let Some(operand) = operand {
                    self.walk_condition(operand);
                }
                for branch in conditions {
                    self.walk_condition(&branch.condition);
                    self.walk(&branch.result, transformation);
                }
                if let Some(result) = else_result {
                    self.walk(result, transformation);
                }
            }
            Expr::Function(function) => {
                // Aggregate values do not constrain a base-row UUID lookup. Their
                // FILTER predicates and nested queries still get independent analysis.
                if !is_aggregate(&function.name) {
                    let name = function.name.to_string();
                    let transform = transformation.unwrap_or_else(|| {
                        let at = function.name.span().start;
                        (&name, (at.line as usize, at.column as usize))
                    });
                    for args in [&function.parameters, &function.args] {
                        if let FunctionArguments::List(list) = args {
                            // Argument labels and ordering are syntax, not transformed inputs.
                            visit_function_args(&list.args, &mut |arg| {
                                self.walk(arg, Some(transform))
                            });
                        }
                    }
                }
                if let Some(filter) = &function.filter {
                    self.walk_condition(filter);
                }
            }
            Expr::Cast {
                expr: value, kind, ..
            } => {
                let transform = transformation.unwrap_or_else(|| {
                    let span = expr.span();
                    let at = super::cast_sites::site(self.tokens, span, kind).unwrap_or(span.start);
                    ("cast", (at.line as usize, at.column as usize))
                });
                self.walk(value, Some(transform));
            }
            Expr::Identifier(_) | Expr::CompoundIdentifier(_)
                if transformation.is_some() && !self.null_test =>
            {
                self.transformed(expr, transformation.unwrap());
            }
            // The shared walker skips scalar subquery columns. Their predicates get their own scope.
            _ => visit_child_exprs(expr, &mut |child| self.walk(child, transformation)),
        }
    }
}
