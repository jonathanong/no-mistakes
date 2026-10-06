use super::expressions::order_prefix_matches_skipping_pinned;
use super::{
    names::{normalize_identifier, normalize_table_name, plain_column},
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
        pinned: &[String],
    ) -> bool {
        self.arbiter_table(table).is_some_and(|table| {
            table
                .indexes
                .iter()
                .filter(|index| index.ordering_supported && index.predicate.is_none())
                .any(|index| {
                    order_prefix_matches_skipping_pinned(order, &index.keys, qualifiers, pinned)
                })
        })
    }
    /// Column sets that hold at most one row per value: valid, ready, live, immediate,
    /// non-partial unique or primary indexes whose keys are all plain columns.
    ///
    /// A partial index only covers some rows and a deferrable one may hold duplicates
    /// inside a transaction, so neither proves uniqueness. Expression keys are not columns.
    pub fn unique_keys(&self, table: &str) -> Vec<Vec<String>> {
        // The same relation `relation()` finds, so a bare name reaches a schema-qualified entry.
        let Some(table) = self
            .relation(table)
            .and_then(|relation| self.arbiter_table(&relation.name))
        else {
            return Vec::new();
        };
        table
            .indexes
            .iter()
            // A key whose operator class or collation is not the column's default may treat
            // values as distinct that an equality on the column treats as equal.
            .filter(|index| {
                index.immediate && index.predicate.is_none() && index.ordering_supported
            })
            .filter_map(|index| {
                index
                    .keys
                    .iter()
                    .map(|key| plain_column(&key.expression))
                    .collect::<Option<Vec<_>>>()
            })
            .collect()
    }
    /// Whether equality on `columns` holds `table` to at most one row: some unique key of
    /// [`Self::unique_keys`] lies wholly among them.
    pub fn columns_pin_one_row(&self, table: &str, columns: &[String]) -> bool {
        let pinned: Vec<String> = columns.iter().map(|c| normalize_identifier(c)).collect();
        self.unique_keys(table)
            .iter()
            .any(|key| !key.is_empty() && key.iter().all(|column| pinned.contains(column)))
    }
    /// Whether `column` of `table` cannot hold NULL. Every row has a `ctid`.
    pub fn column_is_not_null(&self, table: &str, column: &str) -> bool {
        column == "ctid"
            || self.relation(table).is_some_and(|table| {
                table
                    .columns
                    .iter()
                    .any(|candidate| candidate.name == column && !candidate.nullable)
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
