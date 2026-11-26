use super::*;
use crate::tests::args::{TestsArgs, TestsCommand};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: TestsArgs,
}

#[test]
fn changes_matching_reuses_the_prepared_source_store() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/tests-plan-changes");
    let args = Cli::try_parse_from([
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changes",
        "reports/matched.json",
    ])
    .unwrap();
    let TestsCommand::Plan(args) = args.args.command else {
        unreachable!();
    };
    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();
    let input = prepared.changes_input().unwrap();
    let discovered = prepared.discover_tests(TestFramework::Vitest).unwrap();
    for path in &discovered.tests {
        prepared.source_store().read_path(path).unwrap();
    }
    let before = prepared.source_store().physical_read_count();
    let mut plan = TestPlan::default();
    apply(
        &prepared,
        input,
        Some(TestFramework::Vitest),
        discovered,
        &mut plan,
    );
    assert_eq!(plan.selected_tests.len(), 1);
    assert_eq!(prepared.source_store().physical_read_count(), before);
}

#[test]
fn unplanned_changed_file_reason_catches_native_untraceable_warnings() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/tests-plan-changes");
    let args = Cli::try_parse_from([
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changes",
        "reports/matched.json",
    ])
    .unwrap();
    let TestsCommand::Plan(args) = args.args.command else {
        unreachable!();
    };
    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();
    for kind in [
        "swift-dependency-untraceable",
        "dotnet-dependency-untraceable",
    ] {
        let plan = TestPlan {
            warnings: vec![Warning {
                r#type: kind.to_string(),
                message: format!("untraceable {kind}"),
                file: "Package.resolved".to_string(),
                line: None,
            }],
            ..TestPlan::default()
        };
        let discovered = DiscoveredTests {
            tests: Vec::new(),
            targets_by_path: BTreeMap::new(),
            used_fallback: false,
        };
        let reason = unplanned_changed_file_reason(&prepared, None, &discovered, &plan);
        assert_eq!(
            reason.as_deref(),
            Some(format!("untraceable {kind}").as_str())
        );
    }
}
