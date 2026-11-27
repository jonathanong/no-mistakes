use super::identifier_plan::SqlIdentifierScan;
use std::path::Path;

#[derive(Clone)]
#[doc(hidden)]
pub struct ExecutorWarningScan {
    pub(crate) scan: SqlIdentifierScan,
    pub(crate) label: String,
    pub(crate) import_specifier: String,
    pub(crate) omitted: Vec<String>,
    pub(crate) exclusions: Vec<String>,
}

impl ExecutorWarningScan {
    pub(crate) fn includes(&self, root: &Path, path: &Path) -> bool {
        self.scan.includes(root, path)
    }
}
