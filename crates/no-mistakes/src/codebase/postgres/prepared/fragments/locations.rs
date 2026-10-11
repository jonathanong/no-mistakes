use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::fx::FxHashMap;

pub(super) fn collect(
    facts: &SqlStatementFileFacts,
    parsed_sql: &str,
    original_sql: &str,
    origins: &[u32],
) -> FxHashMap<SqlFactSite, usize> {
    let mut out = FxHashMap::default();
    let Some(locations) = &facts.variant_locations else {
        return out;
    };
    let prefix = parsed_sql.len() - original_sql.len();
    let mut tokens = FxHashMap::default();
    let mut at = (1, 1);
    for (offset, character) in parsed_sql.char_indices() {
        tokens.insert(at, offset);
        at = if character == '\n' {
            (at.0 + 1, 1)
        } else {
            (at.0, at.1 + 1)
        };
    }
    for (site, position) in &locations.positions {
        if let Some(offset) = tokens
            .get(&(position.sql_line, position.sql_column))
            .and_then(|offset| offset.checked_sub(prefix))
            .and_then(|offset| origins.get(offset))
        {
            out.insert(site.clone(), *offset as usize);
        }
    }
    out
}
