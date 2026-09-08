use super::expressions::order_prefix_matches_for_qualifiers;
use super::{
    names::{normalize_identifier, normalize_table_name},
    normalize_expression, order_prefix_matches, CanonicalIndex, CanonicalOrderKey, CatalogTable,
    ResolvedArbiter, SchemaCatalog,
};
impl SchemaCatalog {
    pub fn resolve_columns(
        &self,
        table: &str,
        target: &[String],
        predicate: Option<&str>,
    ) -> ResolvedArbiter {
        let Some(table) = self.table(table) else {
            return ResolvedArbiter::Unresolved;
        };
        let mut target = target
            .iter()
            .map(|key| normalize_expression(key))
            .collect::<Vec<_>>();
        target.sort();
        resolve_candidates(
            table
                .indexes
                .iter()
                .filter(|index| {
                    index.predicate.as_deref() == predicate.map(normalize_expression).as_deref()
                })
                .filter(|index| {
                    let mut keys = index
                        .keys
                        .iter()
                        .map(|key| normalize_expression(&key.expression))
                        .collect::<Vec<_>>();
                    keys.sort();
                    keys == target
                })
                .cloned()
                .collect(),
        )
    }
    pub fn resolve_constraint(&self, table: &str, name: &str) -> ResolvedArbiter {
        let Some(table) = self.table(table) else {
            return ResolvedArbiter::Unresolved;
        };
        let name = normalize_identifier(name);
        let direct: Vec<_> = table
            .indexes
            .iter()
            .filter(|index| index.constraint_backed && normalize_identifier(&index.name) == name)
            .cloned()
            .collect();
        if !direct.is_empty() {
            return resolve_candidates(direct);
        }
        let Some(columns) = table.unique_constraints.get(&name) else {
            return ResolvedArbiter::Unresolved;
        };
        resolve_candidates(
            table
                .indexes
                .iter()
                .filter(|index| {
                    index.constraint_backed
                        && index
                            .keys
                            .iter()
                            .map(|key| normalize_expression(&key.expression))
                            .collect::<Vec<_>>()
                            == *columns
                })
                .cloned()
                .collect(),
        )
    }
    pub fn has_canonical_prefix(&self, table: &str, order: &[CanonicalOrderKey]) -> bool {
        self.table(table).is_some_and(|table| {
            table
                .indexes
                .iter()
                .filter(|index| index.predicate.is_none())
                .any(|index| order_prefix_matches(order, &index.keys, true))
        })
    }
    pub(crate) fn has_canonical_prefix_for_qualifiers(
        &self,
        table: &str,
        qualifiers: &[String],
        order: &[CanonicalOrderKey],
    ) -> bool {
        self.table(table).is_some_and(|table| {
            table
                .indexes
                .iter()
                .filter(|index| index.predicate.is_none())
                .any(|index| order_prefix_matches_for_qualifiers(order, &index.keys, qualifiers))
        })
    }
    fn table(&self, table: &str) -> Option<&CatalogTable> {
        self.tables.get(&normalize_table_name(table))
    }
}
fn resolve_candidates(candidates: Vec<CanonicalIndex>) -> ResolvedArbiter {
    let Some(first) = candidates.first() else {
        return ResolvedArbiter::Unresolved;
    };
    if candidates
        .iter()
        .all(|candidate| candidate.keys == first.keys)
    {
        ResolvedArbiter::Exact(first.clone())
    } else {
        ResolvedArbiter::Ambiguous
    }
}
