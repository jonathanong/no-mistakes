//! `ORDER BY` normalization: select aliases, positional references and constant keys.
use crate::codebase::postgres::{
    expression_is_constant, CanonicalOrderKey, SchemaCatalog, SqlConflictInsertFact,
    SqlInsertSourceShape,
};

/// Replace a select alias or an integer position with the select-list expression it names.
pub(super) fn resolve_references(
    order: &[CanonicalOrderKey],
    source: &SqlInsertSourceShape,
) -> Vec<CanonicalOrderKey> {
    order
        .iter()
        .map(|key| CanonicalOrderKey {
            expression: source
                .order_aliases
                .get(&key.expression.to_ascii_lowercase())
                .cloned()
                .or_else(|| positional(&key.expression, source))
                .unwrap_or_else(|| key.expression.clone()),
            ascending: key.ascending,
            nulls_first: key.nulls_first,
        })
        .collect()
}

fn positional(expression: &str, source: &SqlInsertSourceShape) -> Option<String> {
    let position: usize = expression.parse().ok()?;
    source
        .select_list
        .as_ref()?
        .get(position.checked_sub(1)?)
        .cloned()
}

pub(super) fn without_constants(keys: &[CanonicalOrderKey]) -> Vec<CanonicalOrderKey> {
    keys.iter()
        .filter(|key| !expression_is_constant(&key.expression))
        .cloned()
        .collect()
}

/// Whether the source reads one relation pinned to a single row by a catalog unique key.
pub(super) fn pins_one_row(insert: &SqlConflictInsertFact, catalog: &SchemaCatalog) -> bool {
    insert
        .source
        .pinned_relation
        .as_ref()
        .is_some_and(|pinned| catalog.columns_pin_one_row(&pinned.table, &pinned.columns))
}
