use super::{document::Documents, schema, DeclaredPayloadSchema};
use crate::codebase::ts_source::SourceStore;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Component, Path};
use std::sync::Arc;

pub(super) fn load(
    root: &Path,
    reference: &DeclaredPayloadSchema,
    allowed: &BTreeSet<String>,
    sources: &SourceStore,
    documents: &Documents,
) -> Result<schema::Schema, String> {
    let document = read_document(root, reference, allowed, sources, documents)?;
    schema::parse(selected(&document, reference)?, "$", 0)
}

/// Validate document identity and integrity before low may skip schema proof.
pub(super) fn read_document(
    root: &Path,
    reference: &DeclaredPayloadSchema,
    allowed: &BTreeSet<String>,
    sources: &SourceStore,
    documents: &Documents,
) -> Result<Arc<Value>, String> {
    let path = Path::new(&reference.file);
    if reference.file.is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(
            "file must be a nonempty root-relative path without `.` or `..` components".into(),
        );
    }
    if !allowed.contains(&reference.file) {
        return Err(
            "schema document is missing, ignored, or outside the configured rule scope".into(),
        );
    }
    validate_pointer(&reference.pointer)?;
    let document = documents.get(sources, &root.join(path))?;
    Ok(document)
}

pub(super) fn selected<'a>(
    document: &'a Value,
    reference: &DeclaredPayloadSchema,
) -> Result<&'a Value, String> {
    document
        .pointer(&reference.pointer)
        .ok_or_else(|| "JSON pointer does not select a schema".to_string())
}

fn validate_pointer(pointer: &str) -> Result<(), String> {
    if !pointer.is_empty() && !pointer.starts_with('/') {
        return Err("pointer must be empty or an RFC 6901 JSON pointer beginning with `/`".into());
    }
    let mut chars = pointer.chars();
    while let Some(c) = chars.next() {
        if c == '~' && !matches!(chars.next(), Some('0' | '1')) {
            return Err("pointer escape must be `~0` or `~1`".into());
        }
    }
    Ok(())
}

pub(super) fn finding_line(
    root: &Path,
    file: &str,
    allowed: &BTreeSet<String>,
    sources: &SourceStore,
) -> usize {
    // Anchor at the document opening, not a fabricated pointer-specific span.
    // Leading line directives can then suppress unavailable declarations too.
    if allowed.contains(file) {
        if let Ok(source) = sources.read_path(&root.join(file)) {
            return source
                .lines()
                .position(|line| {
                    let line = line.trim();
                    !line.is_empty() && !line.starts_with("//")
                })
                .map_or(1, |index| index + 1);
        }
    }
    1
}
