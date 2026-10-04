//! Catalog-specific lifecycle projection reuses the one prepared SQL fact pass.
use super::super::{bounds, value::PlaceholderPositions};
use super::PreparedStatements;
use crate::codebase::postgres::statements::bounds::TemporaryRelations;
use crate::codebase::postgres::{
    SchemaCatalog, SqlBoundFact, SqlLifecycleBatch, SqlLifecycleFacts, SqlLifecycleStep,
    SqlStatementFileFacts, SqlViewReads,
};
use sqlparser::ast::Statement;
use std::borrow::Cow;

#[derive(Default)]
pub(super) struct LifecycleBuilder {
    facts: SqlLifecycleFacts,
    steps: Vec<SqlLifecycleStep>,
    conditional: bool,
}

impl LifecycleBuilder {
    pub fn collect(
        &mut self,
        statement: &Statement,
        scope: &bounds::Scope,
        positions: PlaceholderPositions<'_>,
        temporary: &TemporaryRelations,
        bounds: &mut Vec<SqlBoundFact>,
    ) -> (usize, Option<SqlViewReads>) {
        let first_bound = bounds.len();
        bounds::collect(statement, scope, positions, bounds);
        let raw = &bounds[first_bound..];
        let view_reads = TemporaryRelations::view_reads(statement, scope, positions);
        self.conditional |= temporary.conditional(statement);
        self.facts.raw_bounds.extend_from_slice(raw);
        self.steps.push(SqlLifecycleStep {
            statement: statement.clone(),
            first_bound,
            last_bound: first_bound + raw.len(),
            view_reads: view_reads.clone(),
        });
        (first_bound, view_reads)
    }

    pub fn finish_batch(&mut self, source: &Statement) {
        self.facts.batches.push(SqlLifecycleBatch {
            source: source.clone(),
            steps: std::mem::take(&mut self.steps),
        });
    }

    pub fn finish(self) -> Option<SqlLifecycleFacts> {
        self.conditional.then_some(self.facts)
    }
}

pub(crate) fn project_bounds<'a>(
    file: &'a SqlStatementFileFacts,
    catalog: &SchemaCatalog,
) -> Cow<'a, [SqlBoundFact]> {
    let Some(lifecycle) = &file.lifecycle else {
        return Cow::Borrowed(&file.bounds);
    };
    let mut bounds = lifecycle.raw_bounds.clone();
    let mut temporary = TemporaryRelations::default();
    let mut prepared = PreparedStatements::default();
    for batch in &lifecycle.batches {
        for step in &batch.steps {
            prepared.apply(
                &batch.source,
                &step.statement,
                &mut temporary,
                &mut bounds[step.first_bound..step.last_bound],
                step.view_reads.as_ref(),
                Some(catalog),
            );
        }
        prepared.record(&batch.source);
    }
    Cow::Owned(bounds)
}
