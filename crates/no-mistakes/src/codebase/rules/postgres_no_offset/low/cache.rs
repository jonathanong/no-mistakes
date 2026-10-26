use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::sql_identifier_syntax::literal_projection::{
    project_literals, LiteralProjection,
};
use crate::codebase::postgres::sql_identifier_syntax::OffsetLiteral;
use std::collections::BTreeMap;
use std::path::Path;

type Span = (u32, u32);
pub(super) fn literals<'a>(
    file: &'a CheckFileFacts,
    facts: &CheckFactMap,
    path: &Path,
    source: &str,
) -> &'a BTreeMap<Span, OffsetLiteral> {
    file.sql_identifier_syntax.offset_literals.get_or_init(|| {
        project_literals(
            file,
            facts,
            path,
            source,
            LiteralProjection {
                project: |statement| statement.offset_uses.clone(),
                parse: |text| crate::codebase::postgres::sql_offset_facts(text).map_err(|_| ()),
                rebase: |fact, base| fact.line += base - 1,
                position: |fact| (fact.line, fact.column),
                parse_metric: "postgres.no_offset.low.literal_parses",
            },
        )
    })
}
