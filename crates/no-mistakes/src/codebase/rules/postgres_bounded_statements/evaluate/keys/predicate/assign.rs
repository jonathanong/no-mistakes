//! One assignment per catalog enum label, capped like OR alternatives.
use super::shape::{Atom, Lit};
use crate::codebase::postgres::{CatalogTable, SchemaCatalog};
use std::collections::{BTreeMap, BTreeSet};

/// Enum label combinations above this stay opaque, matching the OR alternative cap.
const MAX_ASSIGNMENTS: usize = 32;

pub(in super::super) struct Assignment {
    pub(in super::super) proven_empty: bool,
    pub(in super::super) extra: Vec<Atom>,
}

struct Split {
    column: String,
    placeholder: String,
    labels: Vec<String>,
}

pub(super) fn expand(
    params: &[(String, String)],
    bindings: &BTreeMap<String, Lit>,
    conflicts: &BTreeSet<String>,
    table: &CatalogTable,
    catalog: &SchemaCatalog,
) -> Option<Vec<Assignment>> {
    let splits = splits(params, conflicts, table, catalog)?;
    let count = assignment_count(&splits)?;
    Some(
        (0..count)
            .map(|index| assignment(&splits, bindings, index))
            .collect(),
    )
}

fn assignment(splits: &[Split], bindings: &BTreeMap<String, Lit>, mut index: usize) -> Assignment {
    let mut extra = Vec::new();
    let mut proven_empty = false;
    for split in splits {
        let label = &split.labels[index % split.labels.len()];
        index /= split.labels.len();
        let literal = Lit::Str(label.clone());
        if bindings
            .get(&split.placeholder)
            .is_some_and(|bound| bound != &literal)
        {
            proven_empty = true;
        }
        extra.push(Atom::Eq(split.column.clone(), literal));
    }
    Assignment {
        proven_empty,
        extra,
    }
}

fn assignment_count(splits: &[Split]) -> Option<usize> {
    let mut count = 1usize;
    for split in splits {
        count = count.checked_mul(split.labels.len())?;
    }
    (count > 0 && count <= MAX_ASSIGNMENTS).then_some(count)
}

fn splits(
    params: &[(String, String)],
    conflicts: &BTreeSet<String>,
    table: &CatalogTable,
    catalog: &SchemaCatalog,
) -> Option<Vec<Split>> {
    let mut found = Vec::new();
    for (column, placeholder) in params {
        if found
            .iter()
            .any(|split: &Split| split.column == *column && split.placeholder == *placeholder)
        {
            continue;
        }
        if conflicts.contains(placeholder) {
            continue;
        }
        let Some(data_type) = table
            .columns
            .iter()
            .find(|entry| entry.name == *column)
            .map(|entry| entry.data_type.as_str())
        else {
            continue;
        };
        let Some(enum_type) = catalog.enum_type(data_type) else {
            continue;
        };
        found.push(Split {
            column: column.clone(),
            placeholder: placeholder.clone(),
            labels: enum_type.values.clone(),
        });
    }
    (!found.is_empty()).then_some(found)
}
