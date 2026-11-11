use super::*;

pub(super) fn push(
    hits: &mut Vec<RouteHit>,
    file: &Path,
    start: u32,
    line: u32,
    kind: RouteKind,
    detail: &str,
    target: &Path,
) {
    for target in forms(target) {
        hits.push(RouteHit {
            file: file.to_path_buf(),
            start,
            line,
            kind,
            detail: detail.to_string(),
            target,
            symbol: None,
        });
    }
}

impl RouteHit {
    pub(in super::super) fn selected(
        file: &Path,
        site: &crate::codebase::ts_source::module_facts::arguments::RouteSite,
        target: &Path,
        symbol: String,
    ) -> Self {
        Self {
            file: file.to_path_buf(),
            start: site.start,
            line: site.line,
            kind: site.kind,
            detail: site.detail.clone(),
            target: normalize_path(target),
            symbol: Some(symbol),
        }
    }
}
