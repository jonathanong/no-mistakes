use super::*;
use crate::codebase::postgres::statements::SqlStatementFileFacts;
pub(super) fn label(
    record: &SqlStatementFileFacts,
    site: SqlFactSite,
    sql_column: usize,
    path: &Path,
    source: &str,
    sources: &SourceStore,
) -> Result<String> {
    let position = record
        .variant_locations
        .as_ref()
        .and_then(|locations| locations.position(site));
    if let Some(position) = position {
        if let Some(offset) = position.source_offset {
            let foreign = position
                .source_path
                .as_ref()
                .filter(|origin| origin.as_path() != path)
                .map(|origin| sources.read_path(origin))
                .transpose()?;
            if let Some(prefix) = foreign.as_deref().unwrap_or(source).get(..offset) {
                let column = prefix
                    .rsplit('\n')
                    .next()
                    .unwrap_or_default()
                    .chars()
                    .count()
                    + 1;
                return Ok(format!("source column {column}"));
            }
        }
    }
    Ok(format!("SQL column {sql_column}"))
}

#[cfg(test)]
mod tests;
