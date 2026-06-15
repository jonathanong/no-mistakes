use super::generate::{dedupe_warnings, framework_configured};
use super::*;
use crate::config::v2::schema::NoMistakesConfig;
use crate::tests::TestFramework;
use std::collections::BTreeSet;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/impacted-checks/basic"),
    )
}

fn args(files: &[&str]) -> ImpactedChecksArgs {
    ImpactedChecksArgs {
        files: files.iter().map(PathBuf::from).collect(),
        root: fixture(),
        config: None,
        tsconfig: None,
        base: None,
        head: None,
        changed_file: Vec::new(),
        changed_files: None,
        diff: None,
        format: None,
        json: false,
    }
}

fn command_strings(report: &ImpactedChecksReport) -> Vec<String> {
    report.checks.iter().map(|c| c.command.join(" ")).collect()
}

#[test]
fn source_change_yields_test_lint_and_typecheck() {
    let report = generate_impacted_checks(&args(&["src/foo.ts"])).unwrap();
    let commands = command_strings(&report);
    assert!(commands.contains(&"pnpm exec eslint src/foo.ts".to_string()));
    assert!(commands.contains(&"pnpm exec tsc --noEmit".to_string()));
    assert!(commands.contains(&"vitest --project unit src/foo.test.ts".to_string()));
    // prettier (**/*.md) does not match → omitted.
    assert!(!commands.iter().any(|c| c.contains("prettier")));
}

#[test]
fn test_file_change_is_excluded_from_eslint() {
    let report = generate_impacted_checks(&args(&["src/foo.test.ts"])).unwrap();
    let commands = command_strings(&report);
    // eslint excludes src/**/*.test.ts.
    assert!(!commands.iter().any(|c| c.contains("eslint")));
    assert!(commands.contains(&"pnpm exec tsc --noEmit".to_string()));
    assert!(commands.contains(&"vitest --project unit src/foo.test.ts".to_string()));
}

#[test]
fn duplicate_inputs_are_deduped() {
    let mut a = args(&["src/foo.ts"]);
    a.changed_file = vec![PathBuf::from("src/foo.ts")];
    let report = generate_impacted_checks(&a).unwrap();
    let commands = command_strings(&report);
    let unique: BTreeSet<_> = commands.iter().collect();
    assert_eq!(commands.len(), unique.len());
}

#[test]
fn framework_configured_detects_each_source() {
    use crate::config::v2::schema::StringOrList;
    let mut config = NoMistakesConfig::default();
    // All operands evaluate to false.
    assert!(!framework_configured(&config, TestFramework::Vitest));
    assert!(!framework_configured(&config, TestFramework::Playwright));
    assert!(!framework_configured(&config, TestFramework::Swift));
    config.tests.vitest.configs = Some(StringOrList::One("v".to_string()));
    assert!(framework_configured(&config, TestFramework::Vitest));
    config.tests.playwright.configs = Some(StringOrList::One("p".to_string()));
    assert!(framework_configured(&config, TestFramework::Playwright));
    config.tests.swift.packages.push("pkg".to_string());
    assert!(framework_configured(&config, TestFramework::Swift));
}

#[test]
fn renders_every_format() {
    let report = generate_impacted_checks(&args(&["src/foo.ts"])).unwrap();
    // Exercise the report's derived Clone/PartialEq/Debug.
    assert_eq!(report, report.clone());
    assert!(!format!("{report:?}").is_empty());
    assert!(render(&report, Format::Json)
        .unwrap()
        .contains("\"checks\""));
    assert!(render(&report, Format::Yml).unwrap().contains("checks:"));
    assert!(render(&report, Format::Paths)
        .unwrap()
        .contains("vitest --project unit"));
    assert!(render(&report, Format::Md).unwrap().contains("- pnpm exec"));
    assert!(render(&report, Format::Human)
        .unwrap()
        .contains("pnpm exec"));
}

#[test]
fn renders_empty_and_fallback() {
    let report = ImpactedChecksReport {
        changed_files: Vec::new(),
        checks: Vec::new(),
        warnings: Vec::new(),
        fallback_triggered: true,
    };
    let human = render(&report, Format::Human).unwrap();
    assert!(human.contains("No checks for the changed files"));
    assert!(human.contains("fallback triggered"));
}

#[test]
fn run_executes() {
    let mut a = args(&["src/foo.ts"]);
    a.json = true;
    run(a).unwrap();
}

#[test]
fn dedupe_warnings_collapses_and_sorts() {
    let warn = |file: &str| Warning {
        r#type: "dynamic-import".to_string(),
        message: "x".to_string(),
        file: file.to_string(),
    };
    // Two distinct warnings (out of order) plus a duplicate: the dup collapses
    // and the remainder is sorted, exercising the comparator.
    let deduped = dedupe_warnings(vec![warn("b.ts"), warn("a.ts"), warn("b.ts")]);
    assert_eq!(deduped.len(), 2);
    assert_eq!(deduped[0].file, "a.ts");
}
