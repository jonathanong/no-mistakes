use super::{Clause, SqlTransformedColumnUseFact};
use crate::codebase::postgres::idents::{visit_child_exprs, visit_function_args};
use crate::codebase::postgres::statements::select::uses::{columns::bare, scope::BaseRel};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::{Expr, FunctionArguments, Spanned};

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
        match expr {
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
                if let Some((qualifier, column)) = bare(expr, self.positions) {
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
            || rel.table == qualifier || rel.table.rsplit('.').next() == Some(qualifier),
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
