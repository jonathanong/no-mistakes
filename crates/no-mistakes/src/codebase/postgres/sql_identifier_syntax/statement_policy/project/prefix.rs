use super::super::super::Span;
use crate::codebase::postgres::{EmbeddedSqlCall, EmbeddedSqlKind, SqlStatementFileFacts};

/// A complete statement before the first opaque interpolation remains certain.
pub(super) fn complete_statement(
    source: &str,
    at: Span,
    text: &str,
    holes: &[(String, Option<String>)],
    line: u32,
) -> Option<(EmbeddedSqlCall, SqlStatementFileFacts)> {
    let first = holes
        .iter()
        .filter_map(|(marker, _)| text.find(marker))
        .min()?;
    let end = text[..first].rfind(';')? + 1;
    let prefix = text.get(..end)?;
    // JavaScript escapes can change SQL bytes; only an exact authored prefix
    // proves that this statement executes before the unknown remainder.
    let raw = source.get(at.0 as usize + 1..at.0 as usize + 1 + end)?;
    if source.as_bytes().get(at.0 as usize) != Some(&b'`') || raw != prefix {
        return None;
    }
    let facts =
        crate::codebase::postgres::statements::extract_sql_statement_facts_with_location_demand(
            prefix,
            false,
            &[],
            crate::codebase::postgres::statement_facts::SqlLocationDemand::Full,
        );
    if facts.parse_failed {
        return None;
    }
    Some((
        EmbeddedSqlCall {
            line,
            declaration_line: Some(crate::codebase::ts_source::byte_offset_to_line(
                source,
                at.0 as usize,
            )),
            kind: EmbeddedSqlKind::Dynamic,
            sql_text: Some(prefix.to_string()),
            ..Default::default()
        },
        facts,
    ))
}
