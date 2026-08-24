use super::fallback::{fallback_plan, FallbackRequest};
use super::native_semantic_seeds::NativeSemanticSeedResult;
use super::{attach_targets, TestPlan, Warning};
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use std::path::{Path, PathBuf};

#[allow(clippy::too_many_arguments)]
pub(super) fn native_semantic_fallback_plan(
    root: &Path,
    all_tests: &[PathBuf],
    seeds: &NativeSemanticSeedResult,
    enabled: bool,
    limit: usize,
    has_limit: bool,
    warnings: &[Warning],
    discovered_tests: &DiscoveredTests,
) -> Option<TestPlan> {
    if !enabled {
        return None;
    }
    let file = seeds.first_untraceable()?;
    let changed_file = root.join(file);
    let mut plan = fallback_plan(
        root,
        all_tests,
        FallbackRequest {
            group_type: "dependencies",
            via: "native dependency",
            changed_file: Some(&changed_file),
            limit,
            has_limit,
            reason: format!(
                "`{file}` changed a native dependency without a causal test path; falling back to full test suite"
            ),
        },
    );
    plan.warnings.extend(warnings.iter().cloned());
    attach_targets(&mut plan, root, discovered_tests);
    Some(plan)
}
