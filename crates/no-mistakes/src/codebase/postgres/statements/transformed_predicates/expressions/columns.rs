use super::{bare, owner, qualifier, Inputs, SqlTransformedColumnUseFact};
use sqlparser::ast::{Expr, Spanned};

impl Inputs<'_, '_, '_> {
    pub(super) fn transformed(&mut self, expr: &Expr, transformation: &str) {
        let Some((_, column)) = bare(expr, self.positions) else {
            return;
        };
        let qualifier = qualifier(expr);
        let driving_probe = self
            .probe_value
            .filter(|_| {
                self.probed.is_none()
                    || (qualifier.is_some()
                        && owner(self.probed.unwrap(), qualifier.as_deref())
                            .0
                            .is_empty()
                        && !owner(self.relations, qualifier.as_deref()).0.is_empty())
            })
            .map(|index| self.probes[index].clone());
        let (table, candidate_tables) = owner(self.relations, qualifier.as_deref());
        self.out.push(SqlTransformedColumnUseFact {
            table,
            column,
            candidate_tables,
            clause: self.clause,
            line: expr.span().start.line.max(1) as usize,
            column_position: expr.span().start.column.max(1) as usize,
            transformation: transformation.into(),
            driving_probe,
        });
    }
}
