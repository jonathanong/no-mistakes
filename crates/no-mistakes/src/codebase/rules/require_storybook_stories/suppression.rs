use super::types::Component;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::matching_disable_directive;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub(super) fn component_is_suppressed(
    root: &Path,
    sources: &HashMap<std::path::PathBuf, Arc<str>>,
    component: &Component,
) -> bool {
    let component_path = normalize_path(&component.file);
    let rooted_component_path = normalize_path(&root.join(&component.file));
    sources
        .get(&component_path)
        .or_else(|| sources.get(&rooted_component_path))
        .map(Arc::as_ref)
        .is_some_and(|source| {
            matching_disable_directive(source, Some(component.line as u32), super::RULE_ID)
                .is_some()
        })
}

/// Index only selected components from the caller's authoritative fact map.
pub(super) fn component_suppression_sources(
    root: &Path,
    components: &[Component],
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> HashMap<std::path::PathBuf, Arc<str>> {
    components
        .iter()
        .map(|component| &component.file)
        .filter_map(|path| {
            let candidate = normalize_path(&if path.is_absolute() {
                path.clone()
            } else {
                root.join(path)
            });
            let source = shared.ts.get(&candidate)?.source.as_ref().map(Arc::clone)?;
            let normalized = normalize_path(path);
            let rooted = normalize_path(&root.join(path));
            Some((normalized, source, rooted))
        })
        .fold(HashMap::new(), |mut by_path, (path, source, rooted)| {
            by_path.insert(path, Arc::clone(&source));
            by_path.insert(rooted, source);
            by_path
        })
}
