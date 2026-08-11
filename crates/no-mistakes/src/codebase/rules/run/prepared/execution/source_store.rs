use super::super::PreparedRulesCheck;

/// Seed the caller's request session when a legacy prepared request omitted
/// sources, preserving the session as the one source-store owner.
pub(super) fn for_request(
    inputs: &PreparedRulesCheck<'_>,
) -> std::sync::Arc<crate::codebase::ts_source::SourceStore> {
    let snapshot =
        std::sync::Arc::new(crate::codebase::ts_source::VisiblePathSnapshot::from_paths(
            inputs.root,
            inputs.shared.files(),
        ));
    inputs.session.insert_visible_paths(inputs.root, snapshot);
    inputs
        .session
        .visible_paths(inputs.root)
        .source_store_for(inputs.root)
}
