use crate::codebase::postgres::SqlStatementFileFacts;
use std::collections::BTreeMap;

pub(super) fn group(
    statements: &[SqlStatementFileFacts],
) -> BTreeMap<usize, Vec<&SqlStatementFileFacts>> {
    let mut grouped: BTreeMap<usize, Vec<&SqlStatementFileFacts>> = BTreeMap::new();
    for (index, statement) in statements.iter().filter_map(|statement| {
        statement
            .variant_locations
            .as_ref()
            .map(|locations| (locations.call_index, statement))
    }) {
        grouped.entry(index).or_default().push(statement);
    }
    grouped
}
