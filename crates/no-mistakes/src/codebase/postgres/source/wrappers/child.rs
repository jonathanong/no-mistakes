use super::*;
impl<'a, 's> Context<'a, 's> {
    pub(in crate::codebase::postgres::source) fn new(
        source: &'s PostgresSqlSource,
        locations: &'a Locations<'s>,
        recursive_views: &'a RecursiveViews,
        fetch: &'a [crate::codebase::postgres::parse::fetch_expression::Clause],
        generated: &'a [Span],
        markers: &'a [sqlparser::tokenizer::Location],
    ) -> Self {
        Self {
            source,
            locations,
            recursive_views,
            fetch,
            generated,
            markers,
        }
    }
}
impl Context<'_, '_> {
    pub(super) fn statement(
        &self,
        statement: &Statement,
        tokens: &[TokenWithSpan],
        ordinal: usize,
        depth: usize,
        insert_facts: Option<&super::super::insert::parsing::ConflictFacts>,
    ) -> Option<PostgresSqlStatement> {
        let span = span(tokens, self)?;
        let facts = if let Some(wrapper) = project(statement, tokens, self, depth, insert_facts) {
            PostgresSqlStatementKind::Wrapper { wrapper }
        } else if let Statement::Insert(insert) = statement {
            PostgresSqlStatementKind::Insert {
                insert: Box::new(super::super::insert::project(
                    insert,
                    insert_facts,
                    self.locations,
                )),
            }
        } else {
            let tables =
                crate::codebase::postgres::statements::TableTokenIndex::from_iter(tokens.iter());
            super::super::projection::project(
                statement,
                self.locations,
                &tables,
                self.recursive_views,
            )
        };
        let mut result = PostgresSqlStatement {
            ordinal,
            sql: self.source.sql[span.start.offset..span.end.offset].to_string(),
            span,
            facts,
        };
        finalize(&mut result.facts, &result.span);
        Some(result)
    }
}
