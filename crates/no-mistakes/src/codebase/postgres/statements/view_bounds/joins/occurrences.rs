use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlRelationPredicateFact,
};
use crate::codebase::postgres::SchemaCatalog;
use crate::fx::FxHashMap;

type Lines = FxHashMap<usize, Option<usize>>;
type Aliases = FxHashMap<String, Lines>;
pub(super) struct Occurrences(FxHashMap<String, Aliases>);

impl Occurrences {
    pub(super) fn new(
        relations: &[&mut SqlRelationPredicateFact],
        catalog: &SchemaCatalog,
    ) -> Self {
        let mut index: FxHashMap<String, Aliases> = FxHashMap::default();
        for (ordinal, relation) in relations.iter().enumerate() {
            let Some(table) = catalog.bounded_relation(&relation.table) else {
                continue;
            };
            index
                .entry(table.name.clone())
                .or_default()
                .entry(relation.alias.clone().unwrap_or_default())
                .or_default()
                .entry(relation.line)
                .and_modify(|value| *value = None)
                .or_insert(Some(ordinal));
        }
        Self(index)
    }

    pub(super) fn matching(&self, item: &SqlBoundItem, catalog: &SchemaCatalog) -> Option<usize> {
        let SqlBoundItemKind::Table(name) = &item.kind else {
            return None;
        };
        let table = catalog.bounded_relation(name)?;
        let alias = if item.alias_explicit {
            item.alias.as_deref().unwrap_or_default()
        } else {
            ""
        };
        self.0
            .get(&table.name)?
            .get(alias)?
            .get(&item.line)
            .copied()
            .flatten()
    }
}
