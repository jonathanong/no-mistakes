use crate::tests::args::TestFramework;
use no_mistakes::codebase::dependencies::graph::EdgeKind;

/// Browser-only grouping must not discard genuine integration route
/// dependencies from the actual runner's Dependencies group.
pub(super) fn is_browser_coverage(framework: TestFramework, edges: &[EdgeKind]) -> bool {
    framework == TestFramework::Playwright
        && edges.iter().any(|edge| {
            matches!(
                edge,
                EdgeKind::RouteTest | EdgeKind::Layout | EdgeKind::Selector
            )
        })
}
