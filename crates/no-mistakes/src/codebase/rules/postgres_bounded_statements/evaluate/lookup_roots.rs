//! Project keyed lookup joins from the prepared statement pins and catalog keys.
use super::{keys, qualified, reads_outer, Evaluation};
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::SchemaCatalog;
use std::collections::BTreeMap;

pub(super) fn suppressed(
    query: &SqlBoundQuery,
    nested: &[Option<Evaluation>],
    owners: &BTreeMap<String, usize>,
    catalog: &SchemaCatalog,
) -> Vec<bool> {
    let drivers: Vec<bool> = query
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            clean_driver(item, catalog)
                && match &item.kind {
                    SqlBoundItemKind::Table(_) => true,
                    SqlBoundItemKind::Query(_) => nested[index]
                        .as_ref()
                        .is_some_and(|inner| inner.bounded || !inner.offenders.is_empty()),
                    _ => false,
                }
        })
        .collect();
    let dependencies: Vec<Vec<Vec<Vec<usize>>>> = query
        .items
        .iter()
        .enumerate()
        .map(|(item_index, item)| {
            let SqlBoundItemKind::Table(name) = &item.kind else {
                return Vec::new();
            };
            let Some(table) = catalog.bounded_relation(name) else {
                return Vec::new();
            };
            let mut lookup = item.clone();
            lookup.pins.retain(|pin| {
                pin.column != "ctid"
                    && pin.expression.is_none()
                    && pin.peer_column.is_some()
                    && pin.alternative.is_none()
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
                            && drivers.get(source[0]) == Some(&true))
            });
            if lookup.pins.is_empty() {
                return Vec::new();
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
            complete_choices(&lookup, prepared)
        })
        .collect();
    let mut rooted: Vec<bool> = drivers
        .iter()
        .enumerate()
        .map(|(index, driver)| *driver && dependencies[index].is_empty())
        .collect();
    loop {
        let next: Vec<usize> = dependencies
            .iter()
            .enumerate()
            .filter_map(|(index, sources)| {
                (!rooted[index]
                    && sources.iter().any(|key| {
                        key.iter()
                            .all(|column| column.iter().any(|source| rooted[*source]))
                    }))
                .then_some(index)
            })
            .collect();
        if next.is_empty() {
            break;
        }
        for index in next {
            rooted[index] = true;
        }
    }
    dependencies
        .iter()
        .enumerate()
        .map(|(index, sources)| !sources.is_empty() && rooted[index])
        .collect()
}

fn clean_driver(
    item: &crate::codebase::postgres::statements::SqlBoundItem,
    catalog: &SchemaCatalog,
) -> bool {
    if item.lateral
        || reads_outer(&item.lateral_reads, catalog)
        || qualified::reads_outer(&item.lateral_qualified_reads, catalog)
        || item
            .possible_temporary
            .as_ref()
            .is_some_and(|candidate| candidate.uncertain_lifetime)
    {
        return false;
    }
    match &item.kind {
        SqlBoundItemKind::Table(name) => catalog.bounded_relation(name).is_some(),
        SqlBoundItemKind::Query(_) => true,
        SqlBoundItemKind::Other => true,
        SqlBoundItemKind::Opaque => false,
    }
}

fn complete_choices(item: &SqlBoundItem, prepared: keys::Keys) -> Vec<Vec<Vec<usize>>> {
    let mut choices = None;
    for (branch, alternative) in prepared.into_iter().enumerate() {
        if item.rejected_alternatives.contains(&branch) {
            continue;
        }
        let mut complete = Vec::new();
        for key in alternative {
            if key.is_empty() || key.iter().any(|column| column.is_empty()) {
                continue;
            }
            let mut columns = Vec::new();
            let mut valid = true;
            for column in key {
                let mut peers = Vec::new();
                for source in column.into_iter().filter(|source| !source.is_empty()) {
                    peers.extend(source);
                }
                peers.sort_unstable();
                peers.dedup();
                valid &= !peers.is_empty();
                columns.push(peers);
            }
            if valid {
                complete.push(columns);
            }
        }
        if complete.is_empty() {
            return Vec::new();
        }
        if choices.is_none() {
            choices = Some(complete);
        }
    }
    choices.unwrap_or_default()
}
