use super::BOUNDED_VIEW_MARKER;
mod occurrences;
use crate::codebase::postgres::statements::{
    SqlBoundItemKind, SqlBoundQuery, SqlRelationPredicateFact, SqlSelectFact,
};
use crate::codebase::postgres::SchemaCatalog;
use occurrences::Occurrences;

pub(super) fn project(
    query: &SqlBoundQuery,
    selects: &mut [SqlSelectFact],
    catalog: &SchemaCatalog,
    proof: super::LookupProof,
) {
    let mut relations: Vec<_> = selects
        .iter_mut()
        .flat_map(|select| &mut select.relations)
        .collect();
    let occurrences = Occurrences::new(&relations, catalog);
    walk(query, &mut relations, &occurrences, catalog, proof);
}

fn walk(
    query: &SqlBoundQuery,
    relations: &mut [&mut SqlRelationPredicateFact],
    occurrences: &Occurrences,
    catalog: &SchemaCatalog,
    proof: super::LookupProof,
) -> bool {
    let mut items = query.items.clone();
    let matches: Vec<_> = items
        .iter()
        .map(|item| occurrences.matching(item, catalog))
        .collect();
    let mut bounded: Vec<_> = items
        .iter()
        .zip(&matches)
        .map(|(item, matched)| match &item.kind {
            SqlBoundItemKind::Query(inner) => {
                !item.lateral
                    && item.lateral_reads.is_empty()
                    && item.lateral_qualified_reads.is_empty()
                    && walk(inner, relations, occurrences, catalog, proof)
            }
            SqlBoundItemKind::Other => true,
            SqlBoundItemKind::Opaque => false,
            SqlBoundItemKind::Table(_) => matched.is_some_and(|index| {
                relations[index]
                    .constrained_columns
                    .iter()
                    .any(|column| column == BOUNDED_VIEW_MARKER)
            }),
        })
        .collect();
    if !bounded.iter().any(|bounded| *bounded) {
        return false;
    }
    for (item, matched) in items.iter_mut().zip(&matches) {
        if matches!(item.kind, SqlBoundItemKind::Table(_)) && matched.is_none() {
            item.kind = SqlBoundItemKind::Opaque;
        }
    }
    proof(&mut items, &mut bounded, catalog);
    for (matched, bounded) in matches.iter().zip(&bounded) {
        if let Some(index) = matched.filter(|_| *bounded) {
            let columns = &mut relations[index].constrained_columns;
            if !columns.iter().any(|column| column == BOUNDED_VIEW_MARKER) {
                columns.push(BOUNDED_VIEW_MARKER.to_owned());
            }
        }
    }
    bounded.iter().all(|bounded| *bounded)
}
