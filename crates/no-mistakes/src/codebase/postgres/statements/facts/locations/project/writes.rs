use super::super::Locations;
use super::{Site, SqlStatementFileFacts, SqlVariantLocations};
use crate::codebase::postgres::SqlWriteColumns;

pub(super) fn collect(
    index: &Locations,
    facts: &SqlStatementFileFacts,
    out: &mut SqlVariantLocations,
) {
    let mut used = vec![false; index.writes.len()];
    for (i, write) in facts.writes.iter().enumerate() {
        let candidate = index.writes.iter().enumerate().find(|(at, candidate)| {
            !used[*at]
                && candidate.table == write.table
                && candidate.line == write.line
                && match &write.columns {
                    SqlWriteColumns::Named(columns) => {
                        candidate.columns.iter().map(|(name, _)| name).eq(columns)
                    }
                    _ => candidate.columns.is_empty(),
                }
        });
        candidate
            .into_iter()
            .for_each(|(candidate_index, candidate)| {
                used[candidate_index] = true;
                out.insert(Site::Write(i), candidate.at.0, candidate.at.1);
                for (name, at) in &candidate.columns {
                    out.insert(Site::WriteColumn(i, name.clone()), at.0, at.1);
                }
            });
    }
}
