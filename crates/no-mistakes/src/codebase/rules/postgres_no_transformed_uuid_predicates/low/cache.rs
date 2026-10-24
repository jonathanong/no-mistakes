use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::sql_identifier_syntax::literal_projection::{
    project_literals, LiteralProjection,
};
use crate::codebase::postgres::sql_identifier_syntax::TransformedLiteral;
use std::collections::BTreeMap;
use std::path::Path;

type Span = (u32, u32);
pub(super) fn literals<'a>(
    file: &'a CheckFileFacts,
    facts: &CheckFactMap,
    path: &Path,
    source: &str,
) -> &'a BTreeMap<Span, TransformedLiteral> {
    file.sql_identifier_syntax.transformed_literals.get_or_init(|| project_literals(
        file, facts, path, source,
        LiteralProjection {
            project: |statement| statement.transformed_column_uses.clone(),
            parse: |text| {
                let statement = crate::codebase::postgres::statements::extract_sql_statement_facts_with_bounds(text, false);
                if statement.parse_failed { Err(()) } else { Ok(statement.transformed_column_uses) }
            },
            rebase: |fact, base| fact.line += base - 1,
            parse_metric: "postgres.no_transformed_uuid_predicates.low.literal_parses",
        },
    ))
}
