use super::*;

impl ImportClassification {
    pub(crate) fn from_parts(
        resolver_target: Option<PathBuf>,
        workspace_target: Option<PathBuf>,
        workspace_recognized: bool,
    ) -> Self {
        Self {
            resolver_target,
            workspace_target,
            workspace_recognized,
        }
    }
}
