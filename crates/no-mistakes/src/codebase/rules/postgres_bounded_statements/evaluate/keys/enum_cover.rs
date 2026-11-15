//! Each enum label keeps the source dependencies of an index that pins it.
//! See docs/rules/postgres-bounded-statements-proof.md.
use super::super::predicate;
use super::super::{KeyChoices, KeyColumns};
use super::{pinned_key, Pins};
use crate::codebase::postgres::SchemaCatalog;

const MAX_PROOFS: usize = 32;

pub(super) fn keys(
    name: &str,
    pins: &Pins<'_>,
    alternative: usize,
    catalog: &SchemaCatalog,
) -> Option<KeyChoices> {
    let assignments = predicate::assignments(pins.item, alternative, pins.table, catalog)?;
    let indexes: Vec<_> = catalog
        .bounded_unique_indexes(name)
        .iter()
        .filter(|index| index.predicate.is_empty() || pins.item.column_aliases.is_empty())
        .collect();
    let mut choices = vec![Vec::new()];
    for assignment in &assignments {
        if assignment.proven_empty {
            continue;
        }
        let proofs: Vec<_> = indexes
            .iter()
            .filter_map(|index| {
                predicate::holds(
                    &index.predicate,
                    pins.item,
                    alternative,
                    &assignment.extra,
                    pins.table,
                    catalog,
                )
                .then(|| pinned_key(index, pins, alternative))
                .flatten()
                .filter(|key| key.iter().all(|column| !column.is_empty()))
            })
            .collect();
        if proofs.is_empty() {
            return None;
        }
        choices = combine(&choices, &proofs)?;
    }
    Some(choices)
}

fn combine(choices: &[KeyColumns], proofs: &[KeyColumns]) -> Option<KeyChoices> {
    if choices.len().saturating_mul(proofs.len()) > MAX_PROOFS {
        return None;
    }
    let mut combined = Vec::new();
    for choice in choices {
        for proof in proofs {
            let mut key = choice.clone();
            key.extend(proof.clone());
            combined.push(key);
        }
    }
    Some(combined)
}

#[cfg(test)]
#[path = "enum_cover_tests.rs"]
mod tests;
