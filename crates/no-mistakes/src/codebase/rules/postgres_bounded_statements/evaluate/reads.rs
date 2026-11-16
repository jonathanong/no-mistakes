use super::super::view_bounds::{ViewBoundMode, ViewBounds};
use super::{Evaluation, Offender};
use crate::codebase::postgres::{
    CatalogObjectRef, SchemaCatalog, SqlBoundInputMode, SqlBoundItemKind, SqlBoundQuery,
};
use std::collections::HashMap;

pub(super) fn proven_capped(query: &SqlBoundQuery, catalog: &SchemaCatalog) -> bool {
    query.capped
        || query
            .catalog_limit
            .as_ref()
            .is_some_and(|(table, column)| limit_column_not_null(catalog, table, column))
}

/// `ctid` belongs to a base table. A view may project a nullable column of that name.
fn limit_column_not_null(catalog: &SchemaCatalog, table: &str, column: &str) -> bool {
    if column == "ctid" {
        return catalog.bounded_relation(table).is_some();
    }
    catalog.column_is_not_null(table, column)
}

/// Row caps suppress streaming reads, while blocking set inputs remain mandatory.
pub(super) fn collect(
    query: &SqlBoundQuery,
    nested: &[Option<Evaluation>],
    pin_subqueries: &[Vec<Option<Evaluation>>],
    (bounded, credits): (&[bool], &[Option<super::keys::Credit>]),
    catalog: &SchemaCatalog,
    views: &ViewBounds,
    precise_columns: bool,
) -> Vec<Offender> {
    // EXCEPT, INTERSECT, and duplicate-eliminating UNION read every input row.
    let capped = query.input_mode != SqlBoundInputMode::Blocking
        && (row_capped(query, bounded) || proven_capped(query, catalog));
    let mut found = Vec::new();
    for (index, item) in query.items.iter().enumerate() {
        for inner in pin_subqueries[index]
            .iter()
            .flatten()
            .chain(nested[index].iter())
        {
            found.extend(
                inner
                    .offenders
                    .iter()
                    .filter(|offender| !capped || offender.blocking_input)
                    .cloned(),
            );
        }
        if !capped && !bounded[index] && nested[index].is_none() {
            if let SqlBoundItemKind::Table(name) = &item.kind {
                if let Some(mut offender) =
                    relation_offender(name, item.line, item.column, catalog, views)
                {
                    if let Some(super::keys::Credit::Over(product)) =
                        credits.get(index).copied().flatten()
                    {
                        offender.domain_over = Some(product);
                    }
                    found.push(offender);
                }
            }
        }
    }
    let mut positions: HashMap<(String, usize, usize), usize> = HashMap::new();
    let mut out: Vec<Offender> = Vec::new();
    for mut offender in found {
        offender.blocking_input |= query.input_mode == SqlBoundInputMode::Blocking;
        let key = (
            offender.table.clone(),
            offender.line,
            if precise_columns { offender.column } else { 0 },
        );
        if let Some(index) = positions.get(&key) {
            // The same CTE read may occur on streaming and blocking paths: retain either proof.
            out[*index].blocking_input |= offender.blocking_input;
        } else {
            positions.insert(key, out.len());
            out.push(offender);
        }
    }
    out
}

pub(super) fn relation_offender(
    name: &str,
    line: usize,
    column: usize,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Option<Offender> {
    if let Some((view, mode)) = views.get(name, catalog) {
        if mode == ViewBoundMode::Bounded {
            return None;
        }
        return Some(view_offender(&view.name, line, column));
    }
    if let Some(view) = catalog.view_relation(name) {
        return super::view_key::visible_key(catalog, name)
            .map(|_| view_offender(&view.name, line, column));
    }
    let table = catalog.bounded_relation(name)?;
    Some(Offender {
        table: table.name.clone(),
        object: CatalogObjectRef::Table(table.name.clone()),
        line,
        column,
        blocking_input: false,
        domain_over: None,
    })
}

/// `LIMIT`, a pure aggregate, or a plain `GROUP BY` whose keys are already fixed.
pub(super) fn row_capped(query: &SqlBoundQuery, bounded: &[bool]) -> bool {
    query.capped || query.group_constant || grouped(query, bounded)
}

/// Every plain GROUP BY key is one value or comes from an item already bounded.
fn grouped(query: &SqlBoundQuery, bounded: &[bool]) -> bool {
    !query.group_keys.is_empty()
        && query.group_keys.iter().all(|key| {
            key.constant
                || key
                    .item
                    .is_some_and(|item| bounded.get(item).copied().unwrap_or(false))
        })
}

#[cfg(test)]
mod tests;

fn view_offender(name: &str, line: usize, column: usize) -> Offender {
    Offender {
        table: name.to_string(),
        object: CatalogObjectRef::View(name.to_string()),
        line,
        column,
        blocking_input: false,
        domain_over: None,
    }
}
