use crate::codebase::postgres::{SqlCreateIndexMetadata, SqlIndexParam};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) struct LiveIndex<'a> {
    pub(super) rel: String,
    pub(super) path: &'a Path,
    pub(super) index: &'a SqlCreateIndexMetadata,
}

pub(super) fn is_redundant_prefix(shorter: &LiveIndex<'_>, longer: &LiveIndex<'_>) -> bool {
    if shorter.rel == longer.rel
        && shorter.index.line == longer.index.line
        && shorter.index.name == longer.index.name
    {
        return false;
    }
    let (shorter, longer) = (shorter.index, longer.index);
    !shorter.unique
        && shorter.table_name == longer.table_name
        && shorter.access_method == "btree"
        && longer.access_method == "btree"
        && shorter.predicate_key == longer.predicate_key
        && included_columns_subsumed(shorter, longer)
        && is_strict_column_prefix(shorter, longer)
}

fn is_strict_column_prefix(
    shorter: &SqlCreateIndexMetadata,
    longer: &SqlCreateIndexMetadata,
) -> bool {
    !shorter.columns.is_empty()
        && shorter.columns.len() < longer.columns.len()
        && shorter
            .columns
            .iter()
            .zip(longer.columns.iter())
            .all(|(left, right)| same_index_param(left, right))
}

fn same_index_param(left: &SqlIndexParam, right: &SqlIndexParam) -> bool {
    matches!((&left.name, &right.name), (Some(a), Some(b)) if a == b)
        && left.opclass == right.opclass
        && left.ordering == right.ordering
        && left.nulls_ordering == right.nulls_ordering
}

fn included_columns_subsumed(
    shorter: &SqlCreateIndexMetadata,
    longer: &SqlCreateIndexMetadata,
) -> bool {
    if shorter.include_columns.is_empty() {
        return true;
    }
    let longer_columns: BTreeSet<&str> = longer
        .columns
        .iter()
        .filter_map(|column| column.name.as_deref())
        .chain(longer.include_columns.iter().map(String::as_str))
        .collect();
    shorter
        .include_columns
        .iter()
        .all(|column| longer_columns.contains(column.as_str()))
}

pub(super) fn describe_index(index: &SqlCreateIndexMetadata) -> String {
    index.name.clone().unwrap_or_else(|| {
        let columns = index
            .columns
            .iter()
            .map(|column| column.name.as_deref().unwrap_or("<expr>"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("implicit index on ({columns})")
    })
}
