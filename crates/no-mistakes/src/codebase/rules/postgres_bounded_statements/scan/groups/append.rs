//! Attribute appended SQL findings to their executing calls.
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::codebase::rules::LocatedRuleFinding;

pub(super) struct Span {
    call_line: usize,
    opening_shift: i64,
    all_appended: bool,
    relations: Vec<(usize, i64)>,
}

pub(super) fn span(file: &SqlStatementFileFacts, bound_index: usize) -> Option<Span> {
    let locations = file.variant_locations.as_ref()?;
    let opening = locations.position(SqlFactSite::Origin)?;
    let opening_shift = shift(opening.source_line, opening.sql_line);
    let positions: Vec<_> = locations
        .bound_tables
        .iter()
        .filter(|(index, _, _)| *index == bound_index)
        .map(|(_, _, site)| locations.position(site.clone()))
        .collect();
    let all_appended = !locations.append_sites.is_empty()
        && !positions.is_empty()
        && positions.iter().all(|position| {
            position.is_some_and(|position| {
                position.source_path.as_deref().unwrap_or(&file.path)
                    != opening.source_path.as_deref().unwrap_or(&file.path)
                    || shift(position.source_line, position.sql_line) != opening_shift
            })
        });
    let relations = positions
        .into_iter()
        .flatten()
        .filter_map(|position| {
            same_file(position.source_path.as_ref(), &file.path).then_some((
                position.source_line,
                shift(position.source_line, position.sql_line),
            ))
        })
        .collect();
    let recorded = locations.original_call_line as usize;
    let call_line = if recorded > 0 {
        recorded
    } else {
        file.origin_line
    };
    Some(Span {
        call_line,
        opening_shift,
        all_appended,
        relations,
    })
}

pub(super) fn only_appended(spans: &[Span]) -> Option<usize> {
    (!spans.is_empty() && spans.iter().all(|span| span.all_appended))
        .then(|| {
            spans
                .iter()
                .find_map(|span| (span.call_line > 0).then_some(span.call_line))
        })
        .flatten()
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

#[cfg(test)]
mod tests;
