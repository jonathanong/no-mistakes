use super::Sourced;
use crate::codebase::postgres::statements::{SqlBareRead, SqlPinSource, SqlQualifiedRead};
use std::collections::BTreeSet;

/// An `IN` list is sized by the caller, plus whatever other items its elements name.
pub(super) fn merge(sources: Vec<Sourced>) -> Sourced {
    let mut reads: Vec<SqlBareRead> = Vec::new();
    let mut qualified_reads: Vec<SqlQualifiedRead> = Vec::new();
    let mut items: BTreeSet<usize> = BTreeSet::new();
    for sourced in sources {
        reads.extend(sourced.reads);
        qualified_reads.extend(sourced.qualified_reads);
        if let SqlPinSource::Items(named) = sourced.source {
            items.extend(named);
        }
    }
    reads.sort();
    reads.dedup();
    qualified_reads.sort();
    qualified_reads.dedup();
    let source = if items.is_empty() {
        SqlPinSource::Value
    } else {
        SqlPinSource::Items(items.into_iter().collect())
    };
    Sourced {
        source,
        reads,
        qualified_reads,
    }
}
