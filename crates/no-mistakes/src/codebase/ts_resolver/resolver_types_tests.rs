use super::ImportClassification;

impl ImportClassification {
    pub(crate) fn unresolved() -> Self {
        Self {
            resolver_target: None,
            workspace_target: None,
            workspace_recognized: false,
        }
    }
}
