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
    assert!(!invalidation.framework_changed(TestFramework::Jest));
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

fn copied_config_diff(root: &Path) -> DiffFile {
    let mut diff = parse_unified_diff(
        "diff --git a/config-template.yml b/.no-mistakes.yml\nsimilarity index 75%\ncopy from config-template.yml\ncopy to .no-mistakes.yml\n--- a/config-template.yml\n+++ b/.no-mistakes.yml\n@@ -1,3 +1,3 @@\n tests:\n   vitest:\n-    configs: old.ts\n+    configs: new.ts\n",
    )
    .remove(0);
    diff.path = root.join(&diff.path);
    diff.old_path = diff.old_path.map(|path| root.join(path));
    diff
}

#[test]
fn copied_config_reconstructs_from_the_after_checkout_when_the_source_also_changed() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/config-copy/after");
    let diff = copied_config_diff(&root);
    let candidate = root.join(".no-mistakes.yml");

    assert!(
        diff_side_source(&candidate, std::slice::from_ref(&diff), DiffSide::Before)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        diff_side_source(&candidate, std::slice::from_ref(&diff), DiffSide::After)
            .unwrap()
            .unwrap()
            .source,
        "tests:\n  vitest:\n    configs: new.ts\n"
    );
}

#[test]
fn copied_config_reconstructs_the_after_endpoint_from_the_before_checkout() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/config-copy/before");
    let diff = copied_config_diff(&root);
    let candidate = root.join(".no-mistakes.yml");

    assert!(
        diff_side_source(&candidate, std::slice::from_ref(&diff), DiffSide::Before)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        reconstruct_diff_sources(&diff).unwrap(),
        (
            None,
            Some("tests:\n  vitest:\n    configs: new.ts\n".to_string())
        )
    );
}
