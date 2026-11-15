use super::{bare, owner, qualifier, Inputs, SqlTransformedColumnUseFact};
use sqlparser::ast::{Expr, Spanned};

impl Inputs<'_, '_, '_> {
    pub(super) fn transformed(
        &mut self,
        expr: &Expr,
        (transformation, expression_site, provenance): (&str, (usize, usize), &Provenance),
    ) {
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
            bare_anchors: super::super::anchors::for_column(self.anchors, self.relations, expr),
            driving_probe,
            expression: provenance.expression.clone(),
            outer_join_residual: false,
            index_operand: false,
            expression_relations: provenance.references.clone(),
            expression_single_relation: provenance.single_relation,
            expression_site,
        });
    }
}

fn references(
    expression: &Expr,
    relations: &[crate::codebase::postgres::statements::select::uses::scope::BaseRel],
    positions: crate::codebase::postgres::statements::value::PlaceholderPositions<'_>,
) -> (
    Vec<crate::codebase::postgres::statements::SqlTransformedColumnProbeFact>,
    bool,
) {
    let mut references = Vec::new();
    let mut occurrences = std::collections::BTreeSet::new();
    let _: std::ops::ControlFlow<()> = sqlparser::ast::visit_expressions(expression, |expr| {
        if let Some((_, column)) = bare(expr, positions) {
            let qualifier = qualifier(expr);
            let (table, candidate_tables) = owner(relations, qualifier.as_deref());
            if let Some(qualifier) = qualifier {
                let occurrence = relations.iter().find(|relation| {
                    relation.alias.as_deref() == Some(&qualifier)
                        || (relation.alias.is_none() && relation.table == table)
                });
                occurrences.insert(
                    occurrence
                        .map(|relation| relation.alias.as_deref().unwrap_or(&relation.table))
                        .unwrap_or(&qualifier)
                        .to_string(),
                );
            }

            references.push(
                crate::codebase::postgres::statements::SqlTransformedColumnProbeFact {
                    table,
                    column,
                    candidate_tables,
                },
            );
        }
        std::ops::ControlFlow::Continue(())
    });
    (references, occurrences.len() <= 1)
}

pub(super) struct Provenance {
    expression: std::sync::Arc<Expr>,
    references:
        std::sync::Arc<Vec<crate::codebase::postgres::statements::SqlTransformedColumnProbeFact>>,
    single_relation: bool,
}
impl Provenance {
    pub(super) fn new(
        expression: &Expr,
        relations: &[crate::codebase::postgres::statements::select::uses::scope::BaseRel],
        positions: crate::codebase::postgres::statements::value::PlaceholderPositions<'_>,
    ) -> Self {
        let (references, single_relation) = references(expression, relations, positions);
        Self {
            expression: std::sync::Arc::new(expression.clone()),
            references: std::sync::Arc::new(references),
            single_relation,
        }
    }
}
