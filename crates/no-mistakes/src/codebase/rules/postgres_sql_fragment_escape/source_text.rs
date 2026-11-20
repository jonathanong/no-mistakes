use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::SourceStore;
use std::path::Path;
use std::sync::Arc;

pub(super) fn file_text(
    ts: &TsFileFacts,
    sources: Option<&SourceStore>,
    file: &Path,
) -> Option<Arc<str>> {
    ts.source
        .clone()
        .or_else(|| SourceStore::read_optional(sources, file))
}
