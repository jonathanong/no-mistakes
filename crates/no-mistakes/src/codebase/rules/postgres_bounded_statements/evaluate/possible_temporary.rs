//! Project request facts against optional per-catalog search-path evidence.
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery, SqlPinSource};
use crate::codebase::postgres::SchemaCatalog;
use std::borrow::Cow;

pub(super) fn project<'a>(
    query: &'a SqlBoundQuery,
    catalog: &SchemaCatalog,
) -> Cow<'a, SqlBoundQuery> {
    if !has_possible_temporary(query) {
        return Cow::Borrowed(query);
    }
    let mut projected = query.clone();
    resolve(&mut projected, catalog);
    Cow::Owned(projected)
}

fn has_possible_temporary(query: &SqlBoundQuery) -> bool {
    query.items.iter().any(|item| {
        item.possible_temporary.is_some()
            || matches!(&item.kind, SqlBoundItemKind::Query(inner) if has_possible_temporary(inner))
            || item.pins.iter().any(|pin| {
                matches!(&pin.source, SqlPinSource::Query(inner) | SqlPinSource::ReadQuery(inner) if has_possible_temporary(inner))
            })
    })
}

fn resolve(query: &mut SqlBoundQuery, catalog: &SchemaCatalog) {
    for item in &mut query.items {
        let hidden = match (&item.kind, &item.possible_temporary) {
            (SqlBoundItemKind::Table(name), Some(candidate)) => {
                if candidate.uncertain_lifetime {
                    // Either the temporary source survived, or the catalog source did.
                    // Neither branch may lend catalog keys or array lengths to a join.
                    item.pins
                        .retain(|pin| matches!(&pin.source, SqlPinSource::Query(_)));
                    false
                } else {
                    candidate
                        .database_qualifier
                        .as_deref()
                        .is_none_or(|database| catalog.current_database() == Some(database))
                        && catalog.hides_selected_relation(&candidate.earlier_schemas, name)
                }
            }
            _ => false,
        };
        if hidden {
            // A proven temporary source has no permanent catalog pins or array lengths.
            // Its executed pin subqueries remain independent catalog reads.
            item.kind = SqlBoundItemKind::Opaque;
            item.pins.retain(|pin| {
                matches!(
                    &pin.source,
                    SqlPinSource::Query(_) | SqlPinSource::ReadQuery(_)
                )
            });
        }
        if let SqlBoundItemKind::Query(inner) = &mut item.kind {
            resolve(inner, catalog);
        }
        for pin in &mut item.pins {
            if let SqlPinSource::Query(inner) | SqlPinSource::ReadQuery(inner) = &mut pin.source {
                resolve(inner, catalog);
            }
        }
    }
}
