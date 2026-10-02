use super::live::LiveTables;
use crate::codebase::postgres::idents::{parse_relation_identity, resolve_relation_key};

/// Apply configured names to the same final relation that unqualified DML uses.
pub(super) fn relation(tables: &LiveTables<'_>, value: &str) -> Option<String> {
    let key = parse_relation_identity(value)?;
    resolve_relation_key(tables.keys().map(String::as_str), &key)
}
