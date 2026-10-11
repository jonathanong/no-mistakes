use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use std::path::Path;
use std::sync::Arc;

/// Read the fragment's original source through the prepared request store.
pub(super) fn physical_source(
    root: &Path,
    sources: &SourceStore,
    facts: &SqlStatementFileFacts,
    site: &SqlFactSite,
    fallback_rel: &str,
    fallback_source: Option<&Arc<str>>,
) -> (String, Option<Arc<str>>) {
    let Some(path) = facts
        .variant_locations
        .as_ref()
        .and_then(|locations| locations.position(site.clone()))
        .and_then(|position| position.source_path.as_ref())
    else {
        return (fallback_rel.to_string(), fallback_source.cloned());
    };
    (
        relative_slash_path(root, path),
        crate::codebase::rules::read_source(sources, path),
    )
}
