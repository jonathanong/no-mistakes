//! Cut recovered SQL into pieces and join pieces back together, keeping the
//! physical source-line map of every piece exact.

use crate::codebase::postgres::embedded::placeholders::publish_placeholders;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::EmbeddedSqlSourcePosition;
use std::ops::Range;
use std::path::PathBuf;

/// SQL line and column of byte `offset` once generated markers are published.
pub(crate) fn coordinate(sql: &str, offset: usize) -> (u32, u32) {
    let prefix = publish_placeholders(sql[..offset].to_string());
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1;
    let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() as u32 + 1;
    (line, column)
}

/// Physical line (and module, when foreign) that the SQL coordinate reads from.
pub(crate) fn origin_at(
    positions: &[EmbeddedSqlSourcePosition],
    first_line: u32,
    at: (u32, u32),
) -> (u32, Option<PathBuf>) {
    let governing = positions
        .iter()
        .rfind(|position| (position.sql_line, position.sql_column) <= at);
    match governing {
        Some(position) => (
            position.source_line + at.0 - position.sql_line,
            position.source_path.clone(),
        ),
        None => (first_line + at.0 - 1, None),
    }
}

/// Where a cut-out marker was read from.
#[derive(Clone)]
pub(crate) struct Marker {
    pub(crate) line: u32,
    pub(crate) path: Option<PathBuf>,
    /// Physical byte offset of the marker's first character.
    pub(crate) origin: u32,
}

/// Plain SQL text whose first character reads from `marker`.
pub(crate) fn plain(sql: String, marker: Marker) -> RecoveredSql {
    let Marker { line, path, origin } = marker;
    RecoveredSql {
        origins: vec![origin; sql.len()],
        sql,
        line,
        positions: vec![EmbeddedSqlSourcePosition {
            sql_line: 1,
            sql_column: 1,
            source_line: line,
            source_path: path,
        }],
    }
}

/// Pieces between `markers`, each with a self-contained position map, plus the
/// physical origin of every marker. `markers` must be sorted and disjoint.
pub(crate) fn split(
    recovered: &RecoveredSql,
    markers: &[Range<usize>],
) -> (Vec<RecoveredSql>, Vec<Marker>) {
    let mut bounds = Vec::with_capacity(markers.len() + 1);
    let mut start = 0;
    for marker in markers {
        bounds.push(start..marker.start);
        start = marker.end;
    }
    bounds.push(start..recovered.sql.len());
    let pieces = bounds.iter().map(|range| piece(recovered, range)).collect();
    let origins = markers
        .iter()
        .map(|marker| {
            let at = coordinate(&recovered.sql, marker.start);
            let (line, path) = origin_at(&recovered.positions, recovered.line, at);
            let origin = recovered.origins.get(marker.start).copied().unwrap_or(0);
            Marker { line, path, origin }
        })
        .collect();
    (pieces, origins)
}

fn piece(recovered: &RecoveredSql, range: &Range<usize>) -> RecoveredSql {
    let sql = recovered.sql[range.clone()].to_string();
    let from = coordinate(&recovered.sql, range.start);
    let to = coordinate(&recovered.sql, range.end);
    let (line, path) = origin_at(&recovered.positions, recovered.line, from);
    let mut positions: Vec<EmbeddedSqlSourcePosition> = recovered
        .positions
        .iter()
        .filter(|position| {
            let at = (position.sql_line, position.sql_column);
            from <= at && at < to
        })
        .map(|position| EmbeddedSqlSourcePosition {
            sql_line: position.sql_line - from.0 + 1,
            sql_column: if position.sql_line == from.0 {
                position.sql_column - from.1 + 1
            } else {
                position.sql_column
            },
            source_line: position.source_line,
            source_path: position.source_path.clone(),
        })
        .collect();
    if positions
        .first()
        .is_none_or(|first| (first.sql_line, first.sql_column) != (1, 1))
    {
        positions.insert(
            0,
            EmbeddedSqlSourcePosition {
                sql_line: 1,
                sql_column: 1,
                source_line: line,
                source_path: path,
            },
        );
    }
    let mut origins = recovered
        .origins
        .get(range.clone())
        .unwrap_or_default()
        .to_vec();
    origins.resize(sql.len(), 0);
    RecoveredSql {
        origins,
        sql,
        line,
        positions,
    }
}

/// A fragment must open with an explicit entry, otherwise a module change
/// between it and its prefix would be invisible to the position map.
pub(crate) fn with_origin_entry(mut recovered: RecoveredSql) -> RecoveredSql {
    let missing = recovered
        .positions
        .first()
        .is_none_or(|first| (first.sql_line, first.sql_column) != (1, 1));
    if missing {
        recovered.positions.insert(
            0,
            EmbeddedSqlSourcePosition {
                sql_line: 1,
                sql_column: 1,
                source_line: recovered.line,
                source_path: None,
            },
        );
    }
    recovered.origins.resize(recovered.sql.len(), 0);
    recovered
}

/// Concatenates parts in order with the shared append rules (placeholder
/// renumbering and physical-origin tracking). Empty parts contribute nothing.
pub(crate) fn compose(parts: impl IntoIterator<Item = RecoveredSql>) -> Option<RecoveredSql> {
    let mut joined: Option<RecoveredSql> = None;
    for part in parts.into_iter().filter(|part| !part.sql.is_empty()) {
        let part = with_origin_entry(part);
        match &mut joined {
            Some(joined) => joined.append(part),
            None => joined = Some(part),
        }
    }
    joined
}
