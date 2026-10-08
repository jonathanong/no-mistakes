use super::*;
impl<'a, 's> Context<'a, 's> {
    pub(in crate::codebase::postgres::source) fn new(
        source: &'s PostgresSqlSource,
        locations: &'a Locations<'s>,
        recursive_views: &'a RecursiveViews,
        fetch: &'a [crate::codebase::postgres::parse::fetch_expression::Clause],
        generated: &'a [Span],
        markers: &'a [sqlparser::tokenizer::Location],
        comments: super::super::metadata_preparation::Comments,
    ) -> Self {
        Self {
            executes: Default::default(),
            source,
            locations,
            recursive_views,
            fetch,
            generated,
            markers,
            comments: std::cell::RefCell::new(comments),
        }
    }
}
impl Context<'_, '_> {
    pub(in crate::codebase::postgres::source) fn take_prepared(
        &self,
        start: sqlparser::tokenizer::Location,
    ) -> Option<Result<PostgresSqlStatementKind, String>> {
        self.take_execute(start)
            .or_else(|| self.take_comment(start).map(|value| value.0))
    }
    pub(in crate::codebase::postgres::source) fn with_executes(
        mut self,
        executes: super::super::execute_preparation::Occurrences,
    ) -> Self {
        self.executes = std::cell::RefCell::new(executes);
        self
    }
    pub(in crate::codebase::postgres::source) fn take_execute(
        &self,
        start: sqlparser::tokenizer::Location,
    ) -> Option<Result<PostgresSqlStatementKind, String>> {
        self.executes.borrow_mut().remove(&start)
    }
}
impl Context<'_, '_> {
    pub(in crate::codebase::postgres::source) fn statement(
        &self,
        statement: &Statement,
        tokens: &[TokenWithSpan],
        ordinal: usize,
        depth: usize,
        insert_facts: Option<&super::super::insert::parsing::ConflictFacts>,
    ) -> Result<PostgresSqlStatement, String> {
        let span = span(tokens, self).ok_or("Child statement source span is unavailable")?;
        let first = tokens
            .iter()
            .find(|token| !matches!(token.token, Token::Whitespace(_)))
            .unwrap();
        let metadata = self.take_comment(first.span.start);
        let facts = if let Some(comment) = metadata {
            comment.0?
        } else if let Some(wrapper) = project(statement, tokens, self, depth, insert_facts) {
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
        Ok(result)
    }
    pub(in crate::codebase::postgres::source) fn take_comment(
        &self,
        start: sqlparser::tokenizer::Location,
    ) -> Option<(
        Result<PostgresSqlStatementKind, String>,
        sqlparser::tokenizer::Location,
    )> {
        let mut comments = self.comments.borrow_mut();
        let comment = comments.get_mut(&start)?;
        // Move semantic facts once, but keep original boundaries for every ancestor.
        comment.facts.take().map(|facts| (facts, comment.end))
    }
    pub(in crate::codebase::postgres::source) fn source_end(
        &self,
        start: sqlparser::tokenizer::Location,
        end: sqlparser::tokenizer::Location,
    ) -> sqlparser::tokenizer::Location {
        if start > end {
            return end;
        }
        self.comments
            .borrow()
            .range(start..=end)
            .map(|(_, comment)| comment.end)
            .max()
            .unwrap_or(end)
            .max(end)
    }
}
