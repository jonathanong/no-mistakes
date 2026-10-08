use super::{locations::Locations, types::PostgresSqlDiagnostic};

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
