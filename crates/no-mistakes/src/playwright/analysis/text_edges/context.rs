use super::AppTextIndex;
use crate::codebase::dependencies::graph::RouteReachableFiles;
use crate::playwright::analysis::text_types::AppTextTarget;
use crate::playwright::playwright_tests::TestPolicy;

pub(crate) struct TextEdgeContext<'a> {
    pub(crate) app_text_targets: &'a [AppTextTarget],
    pub(crate) app_text_index: &'a AppTextIndex,
    pub(crate) route_reachable_files: &'a RouteReachableFiles,
    pub(crate) test_policy: TestPolicy,
}
