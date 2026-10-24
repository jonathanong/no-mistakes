//! A per-call lateral cap becomes global only after its preceding drivers are bounded.
use super::{qualified, reads_outer};
use crate::codebase::postgres::{decoded_parts, SchemaCatalog, SqlBoundItem, SqlBoundItemKind};

pub(super) fn dependencies(
    item: &SqlBoundItem,
    preceding: &[SqlBoundItem],
    catalog: &SchemaCatalog,
) -> Option<Vec<usize>> {
    // Bare outer references have no retained owner identity. Keep them conservative.
    if item.lateral
        || reads_outer(&item.lateral_reads, catalog)
        || item
            .lateral_reads
            .iter()
            .flat_map(|read| &read.tables)
            .any(|table| catalog.relation(table).is_none())
    {
        return None;
    }
    let mut dependencies = Vec::new();
    for read in &item.lateral_qualified_reads {
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
