use super::sentinel;
use super::split::{compose, split};
use super::unpublish::unpublish;
use crate::codebase::postgres::embedded::placeholders::publish_placeholders_with_positions;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind};
use crate::fx::FxHashMap;

/// Fills every hole of `template` and republishes it as a complete `Composed`
/// call. `holes` maps a hole's site index to its text; any gap yields `None`
/// so the scrubbed `Dynamic` call stays in place.
pub(crate) fn splice(
    template: &EmbeddedSqlCall,
    holes: &FxHashMap<usize, RecoveredSql>,
) -> Option<EmbeddedSqlCall> {
    let published = template.sql_text.as_deref()?;
    let sql = unpublish(published, &template.recovered_placeholder_positions)?;
    let first_line = match template.kind {
        EmbeddedSqlKind::Inline => template.line,
        _ => template.declaration_line.unwrap_or(template.line),
    };
    let caller = RecoveredSql {
        origins: vec![0; sql.len()],
        positions: template.sql_source_positions.clone(),
        line: first_line,
        sql,
    };
    let markers = sentinel::find_all(&caller.sql);
    let ranges = markers
        .iter()
        .map(|(range, _)| range.clone())
        .collect::<Vec<_>>();
    let (pieces, _) = split(&caller, &ranges);
    let mut parts = Vec::with_capacity(pieces.len() * 2);
    let mut pieces = pieces.into_iter();
    parts.extend(pieces.next());
    for ((_, site), piece) in markers.iter().zip(pieces) {
        parts.push(holes.get(site)?.clone());
        parts.push(piece);
    }
    let joined = compose(parts)?;
    let (sql_text, recovered_placeholder_positions) =
        publish_placeholders_with_positions(joined.sql);
    Some(EmbeddedSqlCall {
        sql_text: Some(sql_text),
        kind: EmbeddedSqlKind::Composed,
        declaration_line: Some(joined.line),
        sql_source_positions: joined.positions,
        recovered_placeholder_positions,
        variants: Vec::new(),
        ..template.clone()
    })
}
