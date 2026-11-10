use super::RouteHit;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::module_facts::arguments::RouteKind;
use std::path::Path;

impl RouteHit {
    pub(in crate::codebase::rules::test_no_live_repository_scan) fn nondirect(
        file: &Path,
        detail: &str,
        target: &Path,
    ) -> Self {
        Self {
            file: file.to_path_buf(),
            start: 4,
            line: 8,
            kind: RouteKind::NonDirect,
            detail: detail.to_string(),
            target: normalize_path(target),
        }
    }
}
