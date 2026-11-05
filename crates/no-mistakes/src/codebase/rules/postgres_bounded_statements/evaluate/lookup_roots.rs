//! Project keyed lookup joins from the prepared statement pins and catalog keys.
use super::keys;
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery, SqlPinSource};
use crate::codebase::postgres::SchemaCatalog;
use std::collections::BTreeMap;

pub(super) fn suppressed(
    query: &SqlBoundQuery,
    owners: &BTreeMap<String, usize>,
    catalog: &SchemaCatalog,
) -> Vec<bool> {
    let dependencies: Vec<Option<Vec<usize>>> = query
        .items
        .iter()
        .enumerate()
        .map(|(item_index, item)| {
            let SqlBoundItemKind::Table(name) = &item.kind else {
                return None;
            };
            let table = catalog.bounded_relation(name)?;
            let mut lookup = item.clone();
            lookup.pins.retain(|pin| {
                pin.column != "ctid"
                    && pin.expression.is_none()
                    && pin.peer_column.is_some()
                    && table.columns.iter().any(|column| {
                        !column.nullable
                            && super::super::arrays::key_visible(
                                table,
                                &item.column_aliases,
                                &column.name,
                            ) == Some(pin.column.as_str())
                    })
                    && matches!(&pin.source, SqlPinSource::Items(source)
                        if source.len() == 1
                            && source[0] != item_index
                            && matches!(query.items.get(source[0]).map(|item| &item.kind),
                                Some(SqlBoundItemKind::Table(_) | SqlBoundItemKind::Query(_))))
            });
            if lookup.pins.is_empty() {
                return None;
            }
            let arrays = vec![true; lookup.pins.len()];
            let subqueries = (0..lookup.pins.len()).map(|_| None).collect::<Vec<_>>();
            let prepared = keys::prepare(
                &lookup,
                item_index,
                owners,
                &subqueries,
                &arrays,
                catalog,
                &Default::default(),
            );
            let mut dependencies = Vec::new();
            for alternative in prepared {
                let mut complete = false;
                for key in alternative.into_iter().filter(|key| {
                    !key.is_empty()
                        && key
                            .iter()
                            .all(|column| column.iter().any(|source| !source.is_empty()))
                }) {
                    complete = true;
                    for column in key {
                        for source in column.into_iter().filter(|source| !source.is_empty()) {
                            dependencies.extend(source);
                        }
                    }
                }
                if !complete {
                    return None;
                }
            }
            dependencies.sort_unstable();
            dependencies.dedup();
            (!dependencies.is_empty()).then_some(dependencies)
        })
        .collect();
    (0..dependencies.len())
        .map(|index| {
            dependencies[index].is_some()
                && !reaches(
                    index,
                    index,
                    &dependencies,
                    &mut vec![false; dependencies.len()],
                )
        })
        .collect()
}

fn reaches(
    current: usize,
    target: usize,
    dependencies: &[Option<Vec<usize>>],
    seen: &mut [bool],
) -> bool {
    let Some(sources) = &dependencies[current] else {
        return false;
    };
    for &source in sources {
        if source == target {
            return true;
        }
        if !seen[source] {
            seen[source] = true;
            if reaches(source, target, dependencies, seen) {
                return true;
            }
        }
    }
    false
}
