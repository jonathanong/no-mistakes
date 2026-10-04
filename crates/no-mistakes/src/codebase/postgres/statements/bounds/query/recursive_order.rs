//! PostgreSQL resolves acyclic forward references in a recursive WITH by dependency order.
use super::super::temporary::view_relations;
use crate::codebase::postgres::decoded_parts;
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::Cte;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn indices(ctes: &[Cte]) -> Vec<usize> {
    let aliases = ctes
        .iter()
        .enumerate()
        .map(|(index, cte)| (ident_key(&cte.alias.name), index))
        .collect::<BTreeMap<_, _>>();
    let mut pending = vec![0; ctes.len()];
    let mut dependents = vec![Vec::new(); ctes.len()];
    for (index, cte) in ctes.iter().enumerate() {
        let dependencies = view_relations::names(&cte.query)
            .into_iter()
            .filter_map(|name| {
                let parts = decoded_parts(&name);
                (parts.len() == 1)
                    .then(|| aliases.get(&parts[0]).copied())
                    .flatten()
            })
            .filter(|dependency| *dependency != index)
            .collect::<BTreeSet<_>>();
        pending[index] = dependencies.len();
        for dependency in dependencies {
            dependents[dependency].push(index);
        }
    }
    let mut ready = pending
        .iter()
        .enumerate()
        .filter_map(|(index, count)| (*count == 0).then_some(index))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(ctes.len());
    while let Some(index) = ready.pop_first() {
        order.push(index);
        for dependent in &dependents[index] {
            pending[*dependent] -= 1;
            if pending[*dependent] == 0 {
                ready.insert(*dependent);
            }
        }
    }
    // Mutual recursion is not a supported PostgreSQL CTE form. Keep its projection
    // conservative instead of recursing forever on malformed or unsupported SQL.
    order.extend((0..ctes.len()).filter(|index| pending[*index] != 0));
    order
}
