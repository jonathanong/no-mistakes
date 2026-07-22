use super::*;
use crate::tests::diff_parser::parse_unified_diff;

#[test]
fn reverses_positioned_hunks_against_the_on_disk_version() {
    let diff = parse_unified_diff(
        "diff --git a/.no-mistakes.yml b/.no-mistakes.yml\n--- a/.no-mistakes.yml\n+++ b/.no-mistakes.yml\n@@ -1,3 +1,3 @@\n tests:\n   vitest:\n-    configs: old.ts\n+    configs: new.ts\n",
    );
    assert_eq!(
        apply_unified_hunks("tests:\n  vitest:\n    configs: new.ts\n", &diff[0], true).unwrap(),
        "tests:\n  vitest:\n    configs: old.ts\n"
    );
}

#[test]
fn ignores_deprecated_marker_but_compares_the_selected_framework_only() {
    let mut before = NoMistakesConfig::default();
    before.test_plan.vitest.deprecated_dependencies_key = true;
    let invalidation = ConfigInvalidation {
        comparisons: vec![ConfigComparison {
            before,
            after: NoMistakesConfig::default(),
        }],
        trigger_file: PathBuf::from(".no-mistakes.yml"),
    };
    assert!(!invalidation.framework_changed(TestFramework::Vitest));
    assert!(!invalidation.framework_changed(TestFramework::Playwright));
}

#[test]
fn content_identical_rename_needs_no_hunks_to_reconstruct_its_before_side() {
    let diff = parse_unified_diff(
        "diff --git a/.no-mistakes.yml b/.no-mistakes.yaml\nsimilarity index 100%\nrename from .no-mistakes.yml\nrename to .no-mistakes.yaml\n",
    );
    assert_eq!(
        apply_unified_hunks("tests: {}\n", &diff[0], true).unwrap(),
        "tests: {}\n"
    );
}
