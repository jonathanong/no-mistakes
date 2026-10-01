use super::super::expressions::normalize_expression;
use super::super::names::normalize_identifier;
use super::super::snapshot::SnapshotTable;
use super::super::{ArbiterTable, CanonicalIndex, CanonicalOrderKey};

pub(super) fn arbiter_table(table: &SnapshotTable) -> ArbiterTable {
    let indexes = table
        .indexes
        .iter()
        .filter(|(_, index)| {
            (index.unique || index.primary)
                && index.valid
                && index.ready
                && index.access_method.eq_ignore_ascii_case("btree")
                && !index.keys.is_empty()
        })
        .map(|(name, index)| CanonicalIndex {
            name: name.clone(),
            constraint_backed: index.constraint_backed,
            predicate: index
                .predicate
                .as_ref()
                .map(|value| normalize_expression(value)),
            keys: index
                .keys
                .iter()
                .filter(|key| !key.expression.is_empty())
                .map(|key| CanonicalOrderKey {
                    expression: key.expression.clone(),
                    ascending: !key.descending,
                    nulls_first: key.nulls_first,
                })
                .collect(),
        })
        .filter(|index| !index.keys.is_empty())
        .collect();
    let unique_constraints = table
        .unique_constraints
        .iter()
        .map(|(name, constraint)| {
            (
                normalize_identifier(name),
                constraint
                    .columns
                    .iter()
                    .map(|column| normalize_expression(column))
                    .collect(),
            )
        })
        .collect();
    ArbiterTable {
        indexes,
        unique_constraints,
    }
}
