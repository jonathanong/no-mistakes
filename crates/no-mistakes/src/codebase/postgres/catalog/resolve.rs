use super::expressions::order_prefix_matches_for_qualifiers;
use super::{
    names::{normalize_identifier, normalize_table_name},
    normalize_expression, order_prefix_matches, ArbiterTable, CanonicalIndex, CanonicalOrderKey,
    ResolvedArbiter, SchemaCatalog,
};
impl SchemaCatalog {
    pub fn resolve_columns(
        &self,
        table: &str,
        target: &[String],
        predicate: Option<&str>,
    ) -> ResolvedArbiter {
        let Some(table) = self.arbiter_table(table) else {
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
        let Some(table) = self.arbiter_table(table) else {
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
        self.arbiter_table(table).is_some_and(|table| {
            table
                .indexes
                .iter()
                .filter(|index| index.ordering_supported && index.predicate.is_none())
                .any(|index| order_prefix_matches(order, &index.keys, true))
        })
    }
    pub(crate) fn has_canonical_prefix_for_qualifiers(
        &self,
        table: &str,
        qualifiers: &[String],
        order: &[CanonicalOrderKey],
    ) -> bool {
        self.arbiter_table(table).is_some_and(|table| {
            table
                .indexes
                .iter()
                .filter(|index| index.ordering_supported && index.predicate.is_none())
                .any(|index| order_prefix_matches_for_qualifiers(order, &index.keys, qualifiers))
        })
    }
    fn arbiter_table(&self, table: &str) -> Option<&ArbiterTable> {
        let key = normalize_table_name(table);
        self.tables.get(&key).or_else(|| {
            let schema = self.schema.as_ref()?;
            let schema_identifier = format!("\"{}\"", schema.replace('"', "\"\""));
            let prefix = format!("{}.", normalize_table_name(&schema_identifier));
            self.tables.get(key.strip_prefix(&prefix)?)
        })
    }
}
fn resolve_candidates(candidates: Vec<CanonicalIndex>) -> ResolvedArbiter {
    if candidates
        .iter()
        .any(|index| !index.immediate || !index.ordering_supported)
    {
        return ResolvedArbiter::Unresolved;
    }
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
