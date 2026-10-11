use super::super::super::{SqlFactSite as Site, SqlStatementFileFacts, SqlVariantLocations};
use super::super::Locations;
use super::matches_relation;
use crate::fx::FxHashMap;

pub(super) fn collect(
    index: &Locations,
    facts: &SqlStatementFileFacts,
    out: &mut SqlVariantLocations,
) {
    let mut used = vec![false; index.selections.len()];
    for (i, fact) in facts.selects.iter().enumerate() {
        let node = index
            .selections
            .iter()
            .enumerate()
            .find(|(n, node)| {
                !used[*n]
                    && node.at.0 == fact.line
                    && node.predicate == fact.predicate_sql
                    && fact.tables.iter().all(|table| {
                        node.relations
                            .iter()
                            .any(|relation| &relation.table == table)
                    })
            })
            .map(|(n, node)| {
                used[n] = true;
                node
            });
        let at = node.map_or((fact.line, 1), |node| node.at);
        out.insert(Site::Select(i), at.0, at.1);
        let mut relations = FxHashMap::default();
        for (j, relation) in fact.relations.iter().enumerate() {
            let occurrence = relations
                .entry((relation.table.clone(), relation.alias.clone()))
                .or_insert(0usize);
            let at = node
                .and_then(|node| {
                    node.relations
                        .iter()
                        .filter(|candidate| {
                            candidate.table == relation.table && candidate.alias == relation.alias
                        })
                        .nth(*occurrence)
                })
                .map_or((relation.line, 1), |node| node.at);
            *occurrence += 1;
            out.insert(Site::Relation(i, j), at.0, at.1);
        }
        let mut stars = FxHashMap::default();
        for (j, star) in fact.star_projections.iter().enumerate() {
            let occurrence = stars
                .entry((
                    star.relation.clone(),
                    star.qualified,
                    star.within_function.clone(),
                ))
                .or_insert(0usize);
            let at = node
                .and_then(|node| {
                    node.stars
                        .iter()
                        .filter(|candidate| {
                            candidate.qualifier.is_some() == star.qualified
                                && candidate.function == star.within_function
                                && candidate.qualifier.as_ref().is_none_or(|qualifier| {
                                    matches_relation(&node.relations, &star.relation, qualifier)
                                })
                        })
                        .nth(*occurrence)
                })
                .map_or((star.line, 1), |node| node.at);
            *occurrence += 1;
            out.insert(Site::Star(i, j), at.0, at.1);
        }
        let mut columns = FxHashMap::default();
        for (j, column) in fact.column_uses.iter().enumerate() {
            let occurrence = columns
                .entry((
                    column.table.clone(),
                    column.column.clone(),
                    format!("{:?}", column.clause),
                ))
                .or_insert(0usize);
            let at = node
                .and_then(|node| {
                    node.columns
                        .iter()
                        .filter(|candidate| {
                            candidate.name == column.column
                                && candidate.clause == column.clause
                                && candidate.qualifier.as_ref().is_none_or(|qualifier| {
                                    matches_relation(&node.relations, &column.table, qualifier)
                                })
                        })
                        .nth(*occurrence)
                })
                .map_or((column.line, 1), |node| node.at);
            *occurrence += 1;
            out.insert(Site::Column(i, j), at.0, at.1);
        }
        for (j, (&line, &column)) in fact
            .not_in_subqueries
            .iter()
            .zip(&fact.not_in_columns)
            .enumerate()
        {
            out.insert(Site::NotIn(i, j), line, column);
        }
        for (j, count) in fact.count_existence_checks.iter().enumerate() {
            out.insert(Site::Count(i, j), count.line, count.column);
        }
        for (j, exists) in fact.exists_set_operations.iter().enumerate() {
            out.insert(Site::Exists(i, j), exists.line, exists.column);
        }
    }
}
