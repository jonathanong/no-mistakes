//! A per-call lateral cap becomes global only after its preceding drivers are bounded.
use super::{qualified, reads_outer};
use crate::codebase::postgres::{
    decoded_parts, SchemaCatalog, SqlBareRead, SqlBoundItem, SqlBoundItemKind, SqlQualifiedRead,
};

pub(super) fn dependencies(
    item: &SqlBoundItem,
    preceding: &[SqlBoundItem],
    catalog: &SchemaCatalog,
) -> Option<Vec<usize>> {
    if item.lateral {
        return None;
    }
    read_dependencies(
        &item.lateral_reads,
        &item.lateral_qualified_reads,
        preceding,
        catalog,
    )
}

pub(super) fn read_dependencies(
    bare: &[SqlBareRead],
    qualified_reads: &[SqlQualifiedRead],
    preceding: &[SqlBoundItem],
    catalog: &SchemaCatalog,
) -> Option<Vec<usize>> {
    // Bare outer references have no retained owner identity. Keep them conservative.
    if reads_outer(bare, catalog)
        || bare
            .iter()
            .flat_map(|read| &read.tables)
            .any(|table| catalog.relation(table).is_none())
    {
        return None;
    }
    let mut dependencies = Vec::new();
    for read in qualified_reads {
        if !qualified::reads_outer(std::slice::from_ref(read), catalog) {
            continue;
        }
        // A failed locality proof can mean either an outer read or an unknown local scope.
        // Only complete known local scopes permit matching a preceding driver alias.
        if read.scopes.iter().any(|scope| {
            scope.unknown
                || scope
                    .tables
                    .iter()
                    .any(|table| catalog.relation(table).is_none())
        }) {
            return None;
        }
        let qualifier = decoded_parts(&read.qualifier);
        let mut owners = preceding.iter().enumerate().filter(|(_, source)| {
            if qualifier.len() == 1 && source.alias.as_ref() == qualifier.first() { return true; }
            matches!(&source.kind, SqlBoundItemKind::Table(name) if !source.alias_explicit && decoded_parts(name) == qualifier)
        }).map(|(index, _)| index);
        let owner = owners.next()?;
        if owners.next().is_some() {
            return None;
        }
        dependencies.push(owner);
    }
    dependencies.sort_unstable();
    dependencies.dedup();
    (!dependencies.is_empty()).then_some(dependencies)
}
