use super::*;

pub(super) fn signature_test_facts(root: &Path) -> crate::codebase::ts_source::facts::TsFactMap {
    let files = crate::codebase::ts_source::discover_visible_paths(root);
    crate::codebase::ts_source::facts::collect_ts_facts(
        &files,
        crate::codebase::ts_source::facts::TsFactPlan {
            source: true,
            ..crate::codebase::ts_source::facts::TsFactPlan::imports_and_symbols()
        },
    )
}
