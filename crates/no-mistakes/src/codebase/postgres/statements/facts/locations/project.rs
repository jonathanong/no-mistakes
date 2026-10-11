mod bounds;
mod ordering;
mod selects;
mod tokens;
mod writes;
use super::super::{SqlFactSite as Site, SqlStatementFileFacts, SqlVariantLocations};
use super::{Locations, Relation};
use crate::fx::FxHashMap;
use sqlparser::tokenizer::TokenWithSpan;

pub(super) fn collect(
    index: &Locations,
    facts: &SqlStatementFileFacts,
    tokens: &[TokenWithSpan],
) -> SqlVariantLocations {
    let mut out = SqlVariantLocations::default();
    out.insert(Site::Origin, 1, 1);
    selects::collect(index, facts, &mut out);
    tokens::collect(facts, tokens, &mut out);
    bounds::collect(facts, &mut out);
    ordering::collect(index, tokens, &mut out);
    let mut relation_counts = FxHashMap::default();
    for (groups, update) in [(&facts.updates, true), (&facts.deletes, false)] {
        for (group_index, group) in groups.iter().enumerate() {
            for (relation_index, fact) in group.iter().enumerate() {
                let key = (fact.table.clone(), fact.alias.clone(), fact.line);
                let occurrence = relation_counts.entry(key).or_insert(0usize);
                let at = index
                    .relations
                    .iter()
                    .filter(|candidate| {
                        candidate.table == fact.table
                            && candidate.alias == fact.alias
                            && candidate.at.0 == fact.line
                    })
                    .nth(*occurrence)
                    .map(|candidate| candidate.at)
                    .unwrap_or((fact.line, 1));
                *occurrence += 1;
                out.insert(
                    if update {
                        Site::UpdateRelation(group_index, relation_index)
                    } else {
                        Site::DeleteRelation(group_index, relation_index)
                    },
                    at.0,
                    at.1,
                );
            }
        }
    }
    let mut column_counts = FxHashMap::default();
    for (i, fact) in facts.mutation_column_uses.iter().enumerate() {
        let key = (fact.column.clone(), format!("{:?}", fact.clause), fact.line);
        let occurrence = column_counts.entry(key).or_insert(0usize);
        let at = index
            .mutation_columns
            .iter()
            .filter(|candidate| {
                candidate.name == fact.column
                    && candidate.clause == fact.clause
                    && candidate.at.0 == fact.line
            })
            .nth(*occurrence)
            .map(|candidate| candidate.at)
            .unwrap_or((fact.line, 1));
        *occurrence += 1;
        out.insert(Site::MutationColumn(i), at.0, at.1);
    }
    let mut star_counts = FxHashMap::default();
    for (i, fact) in facts.returning_stars.iter().enumerate() {
        let key = (
            fact.relation.clone(),
            fact.qualified,
            fact.within_function.clone(),
            fact.line,
        );
        let occurrence = star_counts.entry(key).or_insert(0usize);
        let at = index
            .returning
            .iter()
            .filter(|(candidate, relations)| {
                candidate.qualifier.is_some() == fact.qualified
                    && candidate.function == fact.within_function
                    && candidate.at.0 == fact.line
                    && match &candidate.qualifier {
                        Some(qualifier) => matches_relation(relations, &fact.relation, qualifier),
                        None => relations
                            .first()
                            .is_some_and(|relation| relation.table == fact.relation),
                    }
            })
            .nth(*occurrence)
            .map(|(candidate, _)| candidate.at)
            .unwrap_or((fact.line, 1));
        *occurrence += 1;
        out.insert(Site::ReturningStar(i), at.0, at.1);
    }
    let mut inserts = FxHashMap::default();
    for (i, fact) in facts.inserts.iter().enumerate() {
        let occurrence = inserts
            .entry((fact.table.clone(), fact.line))
            .or_insert(0usize);
        let at = index
            .inserts
            .iter()
            .filter(|candidate| candidate.0 == fact.table && candidate.1 .0 == fact.line)
            .nth(*occurrence)
            .map(|candidate| candidate.1)
            .unwrap_or((fact.line, 1));
        *occurrence += 1;
        out.insert(Site::Insert(i), at.0, at.1);
    }
    writes::collect(index, facts, &mut out);
    out
}
fn matches_relation(relations: &[Relation], table: &str, qualifier: &str) -> bool {
    relations.iter().any(|relation| {
        relation.table == table
            && (relation.alias.as_deref() == Some(qualifier)
                || relation.table == qualifier
                || relation.table.rsplit('.').next() == Some(qualifier))
    })
}
