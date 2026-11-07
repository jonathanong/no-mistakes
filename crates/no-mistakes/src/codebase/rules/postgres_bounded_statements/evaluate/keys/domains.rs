use super::super::Evaluation;
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundPin, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::SqlBoundItemKind;
use std::collections::{BTreeMap, BTreeSet};

pub(in super::super) type Domains = BTreeMap<usize, BTreeSet<String>>;

pub(in super::super) fn prepare(query: &SqlBoundQuery, nested: &[Option<Evaluation>]) -> Domains {
    let mut domains = BTreeMap::new();
    for (index, (item, evaluation)) in query.items.iter().zip(nested).enumerate() {
        let (SqlBoundItemKind::Query(inner), Some(evaluation)) = (&item.kind, evaluation) else {
            continue;
        };
        let Some(outputs) = &evaluation.finite_outputs else {
            continue;
        };
        if evaluation.bounded || correlated(item) {
            continue;
        }
        let mut columns = BTreeMap::new();
        for (ordinal, finite) in outputs.iter().enumerate() {
            let Some(name) = exposed_name(inner, ordinal, &item.column_aliases) else {
                continue;
            };
            columns
                .entry(name.to_owned())
                .and_modify(|finite| *finite = false)
                .or_insert(*finite);
        }
        let finite: BTreeSet<_> = columns
            .into_iter()
            .filter_map(|(name, finite)| finite.then_some(name))
            .collect();
        if !finite.is_empty() {
            domains.insert(index, finite);
        }
    }
    domains
}

pub(in super::super) fn output_domains(
    query: &SqlBoundQuery,
    nested: &[Option<Evaluation>],
) -> Option<Vec<bool>> {
    let recursive = query.items.len() == 2
        && query
            .items
            .iter()
            .all(|item| item.alias.is_none() && matches!(item.kind, SqlBoundItemKind::Query(_)))
        && query.outputs.iter().any(|output| output.recursive_domain);
    if !recursive
        && !nested
            .iter()
            .flatten()
            .any(|inner| inner.finite_outputs.is_some())
    {
        return None;
    }
    let outputs: Vec<_> = query
        .outputs
        .iter()
        .enumerate()
        .map(|(ordinal, output)| {
            if output.recursive_domain {
                return recursive
                    && query.items.iter().zip(nested).all(|(item, inner)| {
                        let (SqlBoundItemKind::Query(arm), Some(inner)) = (&item.kind, inner)
                        else {
                            return false;
                        };
                        arm.outputs
                            .get(ordinal)
                            .and_then(|output| output.source_column.as_ref())
                            .and_then(|(source, _)| inner.items.get(*source))
                            .copied()
                            .unwrap_or(false)
                    });
            }
            let Some((source, name)) = &output.source_column else {
                return false;
            };
            let (Some(item), Some(Some(inner))) = (query.items.get(*source), nested.get(*source))
            else {
                return false;
            };
            if correlated(item) {
                return false;
            }
            let (SqlBoundItemKind::Query(source), Some(outputs)) =
                (&item.kind, &inner.finite_outputs)
            else {
                return false;
            };
            let mut matches = outputs.iter().enumerate().filter(|(ordinal, _)| {
                exposed_name(source, *ordinal, &item.column_aliases) == Some(name.as_str())
            });
            let Some((_, finite)) = matches.next() else {
                return false;
            };
            matches.next().is_none() && *finite
        })
        .collect();
    outputs.iter().any(|finite| *finite).then_some(outputs)
}

fn correlated(item: &SqlBoundItem) -> bool {
    item.lateral || !item.lateral_reads.is_empty() || !item.lateral_qualified_reads.is_empty()
}

fn exposed_name<'a>(
    query: &'a SqlBoundQuery,
    ordinal: usize,
    aliases: &'a [String],
) -> Option<&'a str> {
    let output = query.outputs.get(ordinal)?;
    aliases
        .get(ordinal)
        .map(String::as_str)
        .or(output.name.as_deref())
        .or_else(|| output.source_column.as_ref().map(|(_, name)| name.as_str()))
        .or_else(|| {
            if !output.recursive_domain {
                return None;
            }
            let SqlBoundItemKind::Query(anchor) = &query.items.first()?.kind else {
                return None;
            };
            exposed_name(anchor, ordinal, &[])
        })
}

pub(super) fn supplies(pin: &SqlBoundPin, domains: &Domains) -> bool {
    let (SqlPinSource::Items(items), Some(column)) = (&pin.source, &pin.peer_column) else {
        return false;
    };
    let [item] = items.as_slice() else {
        return false;
    };
    domains
        .get(item)
        .is_some_and(|columns| columns.contains(column))
}

#[cfg(test)]
mod tests;
