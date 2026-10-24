use super::{Clause, SqlTransformedColumnUseFact};
use crate::codebase::postgres::statements::select::uses::{columns::bare, scope::BaseRel};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::{Expr, Query, Spanned, Visit, Visitor};
use std::ops::ControlFlow;

pub(super) fn collect(
    expression: &Expr,
    clause: Clause,
    relations: &[BaseRel],
    positions: PlaceholderPositions<'_>,
    out: &mut Vec<SqlTransformedColumnUseFact>,
) {
    let _: ControlFlow<()> = expression.visit(&mut Inputs {
        clause,
        relations,
        positions,
        out,
        query_depth: 0,
        transforms: Vec::new(),
    });
}
struct Inputs<'a, 'b, 'c> {
    clause: Clause,
    relations: &'a [BaseRel],
    positions: PlaceholderPositions<'b>,
    out: &'c mut Vec<SqlTransformedColumnUseFact>,
    query_depth: usize,
    transforms: Vec<String>,
}
impl Visitor for Inputs<'_, '_, '_> {
    type Break = ();
    fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.query_depth += 1;
        ControlFlow::Continue(())
    }
    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.query_depth -= 1;
        ControlFlow::Continue(())
    }
    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if self.query_depth != 0 {
            return ControlFlow::Continue(());
        }
        match expr {
            Expr::Cast { .. } => self.transforms.push("cast".into()),
            Expr::Function(function) => self.transforms.push(function.name.to_string()),
            _ => {}
        }
        if let Some(transformation) = self.transforms.first() {
            // Nested wrappers are visited separately; record only the identifier token.
            if matches!(expr, Expr::Identifier(_) | Expr::CompoundIdentifier(_)) {
                if let Some((qualifier, column)) = bare(expr, self.positions) {
                    let (table, candidate_tables) = owner(self.relations, qualifier.as_deref());
                    self.out.push(SqlTransformedColumnUseFact {
                        table,
                        column,
                        candidate_tables,
                        clause: self.clause,
                        line: expr.span().start.line.max(1) as usize,
                        column_position: expr.span().start.column.max(1) as usize,
                        transformation: transformation.clone(),
                    });
                }
            }
        }
        ControlFlow::Continue(())
    }
    fn post_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if self.query_depth == 0 && matches!(expr, Expr::Cast { .. } | Expr::Function(_)) {
            self.transforms.pop();
        }
        ControlFlow::Continue(())
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
