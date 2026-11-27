use super::{
    locations::Locations,
    types::{PostgresSqlDiagnostic, PostgresSqlStatementKind},
};
use sqlparser::tokenizer::Location;

pub(super) fn append_lexical_error(
    diagnostics: &mut Vec<PostgresSqlDiagnostic>,
    error: Option<sqlparser::tokenizer::TokenizerError>,
    locations: &Locations<'_>,
    source_len: usize,
) {
    if let Some(error) = error {
        diagnostics.push(PostgresSqlDiagnostic {
            message: error.to_string(),
            span: locations
                .position(error.location)
                .map(|position| locations.range(position.offset, source_len)),
        });
    }
}

pub(super) fn unavailable_boundary(
    diagnostics: &mut Vec<PostgresSqlDiagnostic>,
    locations: &Locations<'_>,
    start: Location,
) {
    // Parser compatibility rewrites can introduce synthetic token positions.
    // Do not expose a fabricated source slice when their boundary is unmappable.
    diagnostics.push(PostgresSqlDiagnostic {
        message:
            "Statement source boundary is unavailable after parser compatibility normalization"
                .into(),
        span: locations
            .position(start)
            .map(|position| locations.range(position.offset, position.offset)),
    });
}

pub(super) fn ambiguous_view(
    diagnostics: &mut Vec<PostgresSqlDiagnostic>,
    facts: &PostgresSqlStatementKind,
    span: &crate::source::types::PostgresSqlSpan,
) {
    if matches!(facts, PostgresSqlStatementKind::CreateView { view } if !view.dependencies_complete)
    {
        diagnostics.push(PostgresSqlDiagnostic {
            message: "View TABLE-arm dependencies have ambiguous source identity; dependencies are incomplete".into(),
            span: Some(span.clone()),
        });
    }
}

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
