use super::Columns;
use crate::codebase::postgres::statements::SqlTransformedColumnUseFact;

impl Columns<'_> {
    pub(super) fn anchored(&self, use_: &SqlTransformedColumnUseFact) -> bool {
        use_.bare_anchors.iter().any(|anchor| {
            let Ok((table, _)) = self.resolve_owner(
                &anchor.table,
                &anchor.column,
                anchor.candidate_tables.as_deref(),
            ) else {
                return false;
            };
            let Some(table) = self.catalog.bounded_relation(table) else {
                return false;
            };
            let leading = |columns: &[String]| columns.first() == Some(&anchor.column);
            table.primary_key.as_deref().is_some_and(leading)
                || table
                    .unique_constraints
                    .iter()
                    .any(|key| leading(&key.columns))
                || table.indexes.iter().any(|index| {
                    index.valid
                        && index.ready
                        && index.live
                        && index.predicate.is_none()
                        && index.access_method.eq_ignore_ascii_case("btree")
                        && index
                            .keys
                            .first()
                            .is_some_and(|key| key.column.as_ref() == Some(&anchor.column))
                })
        })
    }
}
