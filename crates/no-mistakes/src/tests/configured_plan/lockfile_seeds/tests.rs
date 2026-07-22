use super::*;
use crate::tests::{Confidence, ImpactReason, TestPlanGroupResult};
use no_mistakes::codebase::test_discovery::{discover_tests, TestRunner};

#[test]
fn post_loop_seed_merges_into_used_targeted_test_at_zero_budget() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/target-scoped-triggers");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = no_mistakes::codebase::ts_resolver::normalize_path(fixture.path());
    let config = no_mistakes::config::v2::load_v2_config(&root, None).unwrap();
    let discovered = discover_tests(&root, &config, TestRunner::Vitest).unwrap();
    let test_path = root.join("src/shared.test.ts");
    let database_target = discovered.targets_by_path[&test_path]
        .iter()
        .find(|target| target.project.as_deref() == Some("database"))
        .unwrap()
        .clone();
    let self_reason = ImpactReason {
        changed_file: "src/shared.test.ts".to_string(),
        path: vec!["src/shared.test.ts".to_string()],
        via: vec!["self".to_string()],
    };
    let targeted_reason = ImpactReason {
        changed_file: "migrations/001.sql".to_string(),
        path: vec![
            "migrations/001.sql".to_string(),
            "src/shared.test.ts".to_string(),
        ],
        via: vec!["configured-trigger".to_string()],
    };
    let lockfile_reason = ImpactReason {
        changed_file: "pnpm-lock.yaml".to_string(),
        path: vec!["lodash".to_string(), "src/shared.test.ts".to_string()],
        via: vec!["import".to_string()],
    };
    let mut selected = BTreeMap::from([(
        test_path,
        SelectedTest {
            test_file: "src/shared.test.ts".to_string(),
            confidence: Confidence::High,
            reasons: vec![self_reason.clone(), targeted_reason.clone()],
            targets: vec![database_target],
        },
    )]);
    let mut used = HashSet::from(["src/shared.test.ts".to_string()]);
    let original_groups = vec![TestPlanGroupResult {
        r#type: "direct".to_string(),
        selected: vec!["src/shared.test.ts".to_string()],
        remaining: discovered.tests.len().saturating_sub(1),
        limit: Some(1),
    }];
    let mut groups = original_groups.clone();

    let result = apply_lockfile_seeds(
        &root,
        LockfileSeedResult {
            candidates: vec![SelectedTest {
                test_file: "src/shared.test.ts".to_string(),
                confidence: Confidence::Medium,
                reasons: vec![lockfile_reason.clone()],
                targets: Vec::new(),
            }],
            untraceable_lockfiles: Vec::new(),
        },
        false,
        &discovered.tests,
        1,
        true,
        &mut selected,
        &mut used,
        &mut groups,
        &discovered,
    )
    .unwrap();

    assert!(result.is_none());
    assert_eq!(groups, original_groups);
    assert_eq!(used, HashSet::from(["src/shared.test.ts".to_string()]));
    let selected_test = selected.get_mut(&root.join("src/shared.test.ts")).unwrap();
    assert_eq!(
        selected_test.reasons,
        vec![self_reason, targeted_reason, lockfile_reason]
    );
    assert!(selected_test.targets.is_empty());
    let mut plan = crate::tests::TestPlan {
        selected_tests: vec![selected_test.clone()],
        groups,
        warnings: Vec::new(),
        fallback_triggered: false,
        fallback_reason: None,
    };
    super::super::attach_targets(&mut plan, &root, &discovered);
    assert_eq!(
        plan.selected_tests[0]
            .targets
            .iter()
            .filter_map(|target| target.project.as_deref())
            .collect::<Vec<_>>(),
        vec!["database", "web"]
    );
}
