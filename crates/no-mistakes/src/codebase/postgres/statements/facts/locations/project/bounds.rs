use super::super::super::{SqlFactSite as Site, SqlStatementFileFacts, SqlVariantLocations};
use crate::codebase::postgres::{SqlBoundItemKind, SqlBoundQuery, SqlPinSource};

pub(super) fn collect(facts: &SqlStatementFileFacts, out: &mut SqlVariantLocations) {
    for (i, bound) in facts.bounds.iter().enumerate() {
        out.insert(Site::Bound(i), bound.line, bound.column);
        query(&bound.query, i, &mut 0, out);
    }
}
fn query(query: &SqlBoundQuery, bound: usize, index: &mut usize, out: &mut SqlVariantLocations) {
    for item in &query.items {
        match &item.kind {
            SqlBoundItemKind::Table(table) => {
                let site = Site::BoundRelation(bound, *index);
                *index += 1;
                out.insert(site.clone(), item.line, item.column);
                out.bound_tables.push((bound, table.clone(), site));
            }
            SqlBoundItemKind::Query(inner) => self::query(inner, bound, index, out),
            _ => {}
        }
        for pin in &item.pins {
            if let SqlPinSource::Query(inner) | SqlPinSource::ReadQuery(inner) = &pin.source {
                self::query(inner, bound, index, out);
            }
        }
    }
}
