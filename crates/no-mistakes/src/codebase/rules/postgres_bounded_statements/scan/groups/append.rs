//! Move a grouped finding onto the call when its earliest root is an appended fragment.
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::codebase::rules::LocatedRuleFinding;

pub(super) struct Span {
    call_line: usize,
    opening_shift: i64,
    relations: Vec<(usize, i64)>,
}

pub(super) fn span(file: &SqlStatementFileFacts, bound_index: usize) -> Option<Span> {
    let locations = file.variant_locations.as_ref()?;
    let relations: Vec<_> = locations
        .bound_tables
        .iter()
        .filter_map(|(index, _, site)| {
            if *index != bound_index {
                return None;
            }
            let position = locations.position(site.clone())?;
            same_file(position.source_path.as_ref(), &file.path).then_some((
                position.sql_line,
                position.sql_column,
                position.source_line,
                shift(position.source_line, position.sql_line),
            ))
        })
        .collect();
    let opening_shift = relations.iter().min().map(|(_, _, _, shift)| *shift)?;
    let recorded = locations.original_call_line as usize;
    let call_line = if recorded > 0 {
        recorded
    } else {
        file.origin_line
    };
    Some(Span {
        call_line,
        opening_shift,
        relations: relations
            .into_iter()
            .map(|(_, _, source_line, shift)| (source_line, shift))
            .collect(),
    })
}

/// The call line when every relation on the group's earliest line sits outside the opening template.
pub(super) fn relocate(
    spans: &[Span],
    findings: &[LocatedRuleFinding],
    executor: &str,
) -> Option<usize> {
    let earliest = findings
        .iter()
        .filter(|located| located.finding.file == executor)
        .map(|located| located.finding.line)
        .min()?;
    let on_opening = spans.iter().any(|span| {
        span.relations
            .iter()
            .any(|(line, shift)| *line == earliest && *shift == span.opening_shift)
    });
    if on_opening {
        return None;
    }
    spans
        .iter()
        .any(|span| {
            span.relations
                .iter()
                .any(|(line, shift)| *line == earliest && *shift != span.opening_shift)
        })
        .then(|| {
            spans
                .iter()
                .find_map(|span| (span.call_line > 0).then_some(span.call_line))
        })
        .flatten()
}

fn same_file(path: Option<&std::path::PathBuf>, file: &std::path::Path) -> bool {
    path.is_none_or(|path| path == file)
}

fn shift(source_line: usize, sql_line: usize) -> i64 {
    source_line as i64 - sql_line as i64
}
