use super::boolean::boolean_result;
use super::{Clause, SqlTransformedColumnUseFact};
use crate::codebase::postgres::idents::{
    ident_key, relation_part_key, relation_suffix_key, visit_child_exprs, visit_function_args,
};
use crate::codebase::postgres::statements::bounds::is_aggregate;
use crate::codebase::postgres::statements::select::uses::{columns::bare, scope::BaseRel};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::{BinaryOperator as Op, Expr, FunctionArguments, Spanned};

pub(super) fn collect(
    expression: &Expr,
    clause: Clause,
    relations: &[BaseRel],
    probed: Option<&[BaseRel]>,
    positions: PlaceholderPositions<'_>,
    out: &mut Vec<SqlTransformedColumnUseFact>,
) {
    Inputs {
        clause,
        relations,
        positions,
        probed,
        probe_value: false,
        positive_conjunct: true,
        out,
    }
    .walk(expression, None);
}
struct Inputs<'a, 'b, 'c> {
    clause: Clause,
    relations: &'a [BaseRel],
    probed: Option<&'a [BaseRel]>,
    probe_value: bool,
    positive_conjunct: bool,
    positions: PlaceholderPositions<'b>,
    out: &'c mut Vec<SqlTransformedColumnUseFact>,
}
impl Inputs<'_, '_, '_> {
    fn equality_operand(&mut self, operand: &Expr, other: &Expr) {
        let previous = (self.probe_value, self.positive_conjunct);
        self.positive_conjunct = false;
        self.probe_value = bare(other, self.positions).is_some_and(|_| {
            let qualifier = qualifier(other);
            let (table, _) = owner(self.probed.unwrap(), qualifier.as_deref());
            qualifier.is_some()
                && !table.is_empty()
                && owner(self.relations, qualifier.as_deref()).0 == table
        });
        self.walk(operand, None);
        (self.probe_value, self.positive_conjunct) = previous;
    }
    fn walk_condition(&mut self, expr: &Expr) {
        let previous = self.probe_value;
        self.probe_value = false;
        self.walk(expr, None);
        self.probe_value = previous;
    }
    fn walk(&mut self, expr: &Expr, transformation: Option<&str>) {
        let previous = self.probe_value;
        let positive_conjunct = self.positive_conjunct;
        self.positive_conjunct &= matches!(
            expr,
            Expr::Nested(_)
                | Expr::BinaryOp {
                    op: Op::Eq | Op::And,
                    ..
                }
        );
        if boolean_result(expr) {
            self.probe_value = false;
        }
        self.walk_value(expr, transformation);
        self.probe_value = previous;
        self.positive_conjunct = positive_conjunct;
    }
    fn walk_value(&mut self, expr: &Expr, transformation: Option<&str>) {
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
                    let transform = transformation.unwrap_or(&name);
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
                    self.walk(filter, None);
                }
            }
            Expr::Cast { expr, .. } => self.walk(expr, Some(transformation.unwrap_or("cast"))),
            Expr::Identifier(_) | Expr::CompoundIdentifier(_) if transformation.is_some() => {
                if let Some((_, column)) = bare(expr, self.positions) {
                    let qualifier = qualifier(expr);
                    if self.probe_value
                        && qualifier.is_some()
                        && owner(self.probed.unwrap(), qualifier.as_deref())
                            .0
                            .is_empty()
                        && !owner(self.relations, qualifier.as_deref()).0.is_empty()
                    {
                        return;
                    }
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

fn qualifier(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Nested(expr) => qualifier(expr),
        Expr::CompoundIdentifier(parts) => Some(
            parts[..parts.len() - 1]
                .iter()
                .map(|part| relation_part_key(&ident_key(part)))
                .collect::<Vec<_>>()
                .join("."),
        ),
        _ => None,
    }
}
