use crate::codebase::postgres::{SchemaCatalog, SqlSelectFact, SqlStatementFileFacts};

pub(super) fn bounded(
    select: &SqlSelectFact,
    facts: &SqlStatementFileFacts,
    catalog: Option<&SchemaCatalog>,
) -> bool {
    let Some(columns) = facts.index_probes.get(&select.sql_position) else {
        return false;
    };
    let [relation] = select.relations.as_slice() else {
        return false;
    };
    let Some(table) = catalog.and_then(|catalog| catalog.relation(&relation.table)) else {
        return false;
    };
    table.indexes.iter().any(|index| {
        index.valid
            && index.ready
            && index.live
            && index.predicate.is_none()
            && index
                .keys
                .first()
                .and_then(|key| key.column.as_ref())
                .is_some_and(|column| columns.contains(column))
    })
}
