use super::{SourceDemand, PLAYWRIGHT_NO_SERIAL, TEST_NO_RETRIES};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::rules::filesystem_dispatch::{
    run_filesystem_rules, run_filesystem_rules_with_files,
};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

fn library_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-reliability/library")
}

#[test]
fn filesystem_library_entrypoints_keep_rule_findings_messages_and_suppressions() {
    let root = library_fixture().canonicalize().unwrap();
    let tracked = [
        ".no-mistakes.yml",
        "playwright.config.ts",
        "reliability.spec.ts",
    ]
    .map(|relative| root.join(relative));

    let discovered = run_filesystem_rules(&root, None).unwrap();
    let explicit = run_filesystem_rules_with_files(&root, None, &tracked).unwrap();

    assert_eq!(discovered, explicit);
    let findings: Vec<_> = discovered
        .iter()
        .map(|finding| (finding.file.as_str(), finding.line, finding.rule.as_str()))
        .collect();
    assert_eq!(
        findings,
        [
            ("playwright.config.ts", 3, PLAYWRIGHT_NO_SERIAL),
            ("reliability.spec.ts", 5, PLAYWRIGHT_NO_SERIAL),
            ("reliability.spec.ts", 4, TEST_NO_RETRIES),
        ]
    );
    assert_eq!(
        discovered
            .iter()
            .find(|finding| finding.rule == TEST_NO_RETRIES)
            .unwrap()
            .message,
        "Preserve the first test failure."
    );
}

#[test]
fn filesystem_library_returns_the_unanalyzable_input() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/test-no-retries")
        .canonicalize()
        .unwrap();
    let root = fixture.join("fixture/malformed");
    let config = fixture.join("configs/malformed.yml");
    let error =
        run_filesystem_rules_with_files(&root, Some(&config), &[root.join("broken.spec.ts")])
            .unwrap_err();
    let input = error.downcast_ref::<CheckInputFailure>().unwrap();
    assert_eq!(input.file, Some(PathBuf::from("broken.spec.ts")));
}

#[test]
fn merging_fact_plans_keeps_both_runner_demands_for_a_shared_test() {
    let fixture = library_fixture();
    let shared_test = fixture.join("reliability.spec.ts");
    let config = fixture.join("playwright.config.ts");
    let mut retries = CheckFactPlan {
        test_reliability_paths: Arc::new(BTreeMap::from([(
            shared_test.clone(),
            SourceDemand {
                standard_tests: true,
                ..Default::default()
            },
        )])),
        ..Default::default()
    };
    let serial = CheckFactPlan {
        test_reliability_paths: Arc::new(BTreeMap::from([
            (
                shared_test.clone(),
                SourceDemand {
                    playwright_tests: true,
                    ..Default::default()
                },
            ),
            (
                config.clone(),
                SourceDemand {
                    playwright_config: true,
                    ..Default::default()
                },
            ),
        ])),
        ..Default::default()
    };

    retries.include(serial);

    let paths = &retries.test_reliability_paths;
    assert_eq!(paths.len(), 2);
    assert!(paths[&shared_test].standard_tests);
    assert!(paths[&shared_test].playwright_tests);
    assert!(paths[&config].playwright_config);
}
