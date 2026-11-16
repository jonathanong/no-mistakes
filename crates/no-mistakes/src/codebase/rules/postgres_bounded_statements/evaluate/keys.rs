//! Compile catalog and pin evidence once before bounded-state propagation.
use super::{qualified, reads_outer, Evaluation};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlPinSource};
use crate::codebase::postgres::{RelationKind, SchemaCatalog};
use std::collections::BTreeMap;
mod domain_apply;
mod domain_range;
mod domain_rows;
mod domains;
mod implied;
mod indexes;
mod not_null;
mod predicate;
pub(in crate::codebase::rules::postgres_bounded_statements) use domain_apply::Credit;
pub(super) use domains::{output_domains, prepare as finite_domains, Domains};
mod view;

/// Each pin source requires all its referenced items to be bounded.
type SourceItems = Vec<usize>;
/// Any source can supply a column; every column must complete one catalog key.
type ColumnSources = Vec<SourceItems>;
type KeyColumns = Vec<ColumnSources>;
/// Any complete unique key proves one predicate branch; every branch must be proven.
type KeyChoices = Vec<KeyColumns>;
pub(super) type Keys = Vec<KeyChoices>;

/// One synthetic key whose source requires every proven preceding driver.
pub(super) fn dependent_on(items: SourceItems) -> Keys {
    vec![vec![vec![vec![items]]]]
}

pub(super) fn prepare(
    item: &SqlBoundItem,
    item_index: usize,
    (owners, domains, domain_rows): (&BTreeMap<String, usize>, Option<&Domains>, Option<u64>),
    subqueries: &[Option<Evaluation>],
    arrays: &[bool],
    catalog: &SchemaCatalog,
    views: &super::super::view_bounds::ViewBounds,
) -> (Keys, Option<Credit>) {
    let SqlBoundItemKind::Table(name) = &item.kind else {
        return (Vec::new(), None);
    };
    let Some(table) = catalog.bounded_relation(name) else {
        // viewBounds is the cardinality contract, so a derived key must not override it.
        if views.get(name, catalog).is_some() {
            return (Vec::new(), None);
        }
        return (
            view::keys_for(item, item_index, owners, subqueries, arrays, catalog, name),
            None,
        );
    };
    let mut specs: Vec<domain_rows::Spec> = catalog
        .unique_keys(name)
        .into_iter()
        .filter_map(|key| domain_rows::spec(table, &item.column_aliases, key, true))
        .chain(views.keys.columns(&table.name).filter_map(|column| {
            domain_rows::spec(table, &item.column_aliases, vec![column.to_owned()], false)
        }))
        .collect();
    if table.relation_kind != RelationKind::PartitionedTable
        && !item.column_aliases.iter().any(|alias| alias == "ctid")
    {
        specs.push(domain_rows::Spec {
            columns: vec![("ctid".to_owned(), "ctid".to_owned())],
            eligible: false,
        });
    }
    // Static read/cast evidence is independent of the fixed point, including failed proofs.
    let sources: Vec<_> = item
        .pins
        .iter()
        .enumerate()
        .map(|(index, pin)| {
            if !arrays[index]
                || reads_outer(&pin.reads, catalog)
                || qualified::reads_outer(&pin.qualified_reads, catalog)
            {
                return None;
            }
            match &pin.source {
                SqlPinSource::Value => Some(Vec::new()),
                SqlPinSource::Items(_)
                    if domains.is_some_and(|domains| domains::supplies(pin, domains)) =>
                {
                    Some(Vec::new())
                }
                SqlPinSource::Items(items) | SqlPinSource::Array { items, .. } => {
                    Some(items.clone())
                }
                SqlPinSource::Query(_) => subqueries[index]
                    .as_ref()
                    .filter(|query| query.bounded)
                    .map(|_| Vec::new()),
                SqlPinSource::ArrayQuery(query) => subqueries[index]
                    .as_ref()
                    .filter(|evaluation| {
                        evaluation.bounded && super::scalar_output::single(query, catalog)
                    })
                    .map(|_| Vec::new()),
                SqlPinSource::StoredArray(_) | SqlPinSource::ReadQuery(_) => None,
            }
        })
        .collect();
    let mut prepared: Keys = (0..item.pin_alternatives.max(1))
        .map(|alternative| {
            // A rejected alternative matches no row, so it needs no catalog key.
            if item.rejected_alternatives.contains(&alternative) {
                return vec![vec![vec![vec![]]]];
            }
            specs
                .iter()
                .map(|spec| {
                    spec.columns
                        .iter()
                        .map(|(visible, physical)| {
                            let not_null = physical == "ctid"
                                || table
                                    .columns
                                    .iter()
                                    .any(|column| column.name == *physical && !column.nullable);
                            item.pins
                                .iter()
                                .zip(&sources)
                                .filter_map(|(pin, source)| {
                                    (pin.column == *visible
                                        && pin
                                            .alternative
                                            .is_none_or(|branch| branch == alternative)
                                        && pin.expression.is_none()
                                        && (!pin.unqualified_candidate
                                            || owners.get(&pin.column) == Some(&item_index))
                                        && (!pin.null_safe || not_null))
                                        .then(|| source.clone())
                                        .flatten()
                                })
                                .collect()
                        })
                        .collect()
                })
                .collect()
        })
        .collect();
    let mut widths = domain_rows.map(|_| {
        (0..item.pin_alternatives.max(1))
            .map(|alternative| {
                if item.rejected_alternatives.contains(&alternative) {
                    return vec![vec![None]];
                }
                specs
                    .iter()
                    .map(|spec| {
                        domain_rows::widths_for(
                            table,
                            catalog,
                            item,
                            alternative,
                            &spec.columns,
                            spec.eligible,
                        )
                    })
                    .collect()
            })
            .collect::<domain_rows::StatementWidths>()
    });
    indexes::append(
        &mut prepared,
        item,
        item_index,
        owners,
        (name, table, widths.as_mut()),
        &sources,
        catalog,
    );
    let credit = domain_rows.and_then(|cap| {
        domain_apply::apply(
            &mut prepared,
            widths.as_deref().unwrap_or(&[]),
            &item.rejected_alternatives,
            cap,
        )
    });
    (prepared, credit)
}
