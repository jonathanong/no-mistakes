use super::*;

impl Context<'_, '_> {
    pub(super) fn restore(
        &self,
        statement: &mut Statement,
        end: sqlparser::tokenizer::Location,
    ) -> Result<(), String> {
        self.recursive_views
            .validate(statement)
            .map_err(|error| error.to_string())?;
        self.recursive_views.restore(statement);
        crate::codebase::postgres::parse::fetch_expression::restore(statement, self.fetch);
        generated(statement, self.generated, end);
        Ok(())
    }
}
fn generated(statement: &mut Statement, spans: &[Span], end: sqlparser::tokenizer::Location) {
    super::super::generated::restore(statement, spans, end);
    match statement {
        Statement::Explain { statement, .. } | Statement::Prepare { statement, .. } => {
            generated(statement, spans, end)
        }
        _ => {}
    }
}
