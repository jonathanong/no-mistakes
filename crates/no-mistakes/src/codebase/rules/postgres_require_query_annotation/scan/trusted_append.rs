use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{EmbeddedSqlCall, EmbeddedSqlKind};
use std::path::Path;

pub(super) fn accepts(
    facts: &CheckFactMap,
    path: &Path,
    call: &EmbeddedSqlCall,
    span: (u32, u32),
) -> bool {
    call.kind == EmbeddedSqlKind::Composed
        && call.variants.is_empty()
        && facts
            .ts
            .get(path)
            .and_then(|file| file.query_annotation_heads.as_deref())
            .is_some_and(|heads| heads.accepts_trusted_append(span))
}
