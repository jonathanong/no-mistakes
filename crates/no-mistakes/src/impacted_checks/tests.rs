use super::frameworks::framework_present;
use super::generate::{
    dedupe_checks, dedupe_warnings, generate_impacted_checks_with_stats, generic_checks,
    plan_args_for,
};
use super::*;
use crate::config::v2::schema::NoMistakesConfig;
use crate::tests::TestFramework;
use std::collections::BTreeSet;
use std::path::Path;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/impacted-checks/basic"),
    )
}

fn multi_framework_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/impacted-checks/multi-framework"),
    )
}

fn generic_only_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/impacted-checks/generic-only"),
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
        diff_content: None,
        format: None,
        json: false,
        timings: false,
    }
}

fn fanout_fixture() -> tempfile::TempDir {
    crate::test_support::materialize_gitignore_fixture("test-plan-fanout")
}

fn fanout_args(root: &Path) -> ImpactedChecksArgs {
    ImpactedChecksArgs {
        files: vec![PathBuf::from("src/shared.ts")],
        root: root.to_path_buf(),
        config: None,
        tsconfig: None,
        base: None,
        head: None,
        changed_file: Vec::new(),
        changed_files: None,
        diff: None,
        diff_content: None,
        format: None,
        json: false,
        timings: false,
    }
}

fn command_strings(report: &ImpactedChecksReport) -> Vec<String> {
    report.checks.iter().map(|c| c.command.join(" ")).collect()
}

#[test]
fn multi_file_change_selects_all_configured_frameworks() {
    let mut a = args(&[
        "src/value.ts",
        "dotnet/src/App/Value.cs",
        "swift/App/Sources/App/Value.swift",
    ]);
    a.root = multi_framework_fixture();

    let (report, stats) = generate_impacted_checks_with_stats(&a).unwrap();

    assert_eq!(
        report.changed_files,
        vec![
            "dotnet/src/App/Value.cs".to_string(),
            "src/value.ts".to_string(),
            "swift/App/Sources/App/Value.swift".to_string(),
        ]
    );
    assert_eq!(
        command_strings(&report),
        vec![
            "dotnet test dotnet/tests/App.Tests/App.Tests.csproj --no-restore".to_string(),
            "playwright test --project e2e e2e/value\\.spec\\.ts".to_string(),
            "swift test --package-path swift/App --filter AppTests".to_string(),
            "vitest --project unit src/value.test.ts".to_string(),
        ]
    );
    assert!(report.warnings.is_empty());
    assert!(!report.fallback_triggered);
    assert_eq!(stats.framework_discoveries, 4);
    assert_eq!(stats.graph_builds, 1);
}

#[test]
fn all_environments_skip_the_shared_graph() {
    let mut a = args(&["src/value.ts"]);
    a.root = multi_framework_fixture();
    a.config = Some(a.root.join("all.no-mistakes.yml"));

    let (report, stats) = generate_impacted_checks_with_stats(&a).unwrap();

    assert_eq!(report.checks.len(), 4);
    assert!(report.fallback_triggered);
    assert_eq!(stats.framework_discoveries, 4);
    assert_eq!(stats.graph_builds, 0);
}

#[test]
fn generic_only_repository_skips_test_file_discovery_and_graph() {
    let mut a = args(&["src/value.ts"]);
    a.root = generic_only_fixture();
    // Test-only inputs remain irrelevant when no test framework is present.
    a.tsconfig = Some(a.root.join("missing-tsconfig.json"));

    let (report, stats) = generate_impacted_checks_with_stats(&a).unwrap();

    assert_eq!(command_strings(&report), vec!["eslint src/value.ts"]);
    assert_eq!(stats.framework_discoveries, 0);
    assert_eq!(stats.graph_builds, 0);
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
fn renders_empty_warnings_and_fallback() {
    let report = ImpactedChecksReport {
        changed_files: Vec::new(),
        checks: Vec::new(),
        warnings: vec![Warning {
            r#type: "dynamic-import".to_string(),
            message: "uncertain".to_string(),
            file: "a.ts".to_string(),
        }],
        fallback_triggered: true,
    };
    let human = render(&report, Format::Human).unwrap();
    assert!(human.contains("No checks for the changed files"));
    // Warnings are surfaced in human/md output, not just JSON/YAML.
    assert!(human.contains("warning: a.ts: uncertain"));
    assert!(human.contains("fallback triggered"));
}

#[test]
fn run_executes() {
    let mut a = args(&["src/foo.ts"]);
    a.json = true;
    run(a).unwrap();
}

#[test]
fn dedupe_checks_merges_files_for_same_command() {
    let mk = |files: &[&str]| CheckCommand {
        name: "tsc".to_string(),
        kind: CheckKind::Generic,
        command: vec!["tsc".to_string()],
        files: files.iter().map(|f| f.to_string()).collect(),
    };
    let out = dedupe_checks(vec![mk(&["b.ts"]), mk(&["a.ts"])]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].files, vec!["a.ts".to_string(), "b.ts".to_string()]);
}

#[test]
fn generic_checks_excludes_deleted_from_append() {
    use crate::config::v2::schema::{CheckCommandDef, CheckFileArgs};
    let mut config = NoMistakesConfig::default();
    config.checks.commands = vec![
        CheckCommandDef {
            name: "eslint".to_string(),
            include: vec!["**/*.ts".to_string()],
            command: vec!["eslint".to_string()],
            file_args: CheckFileArgs::Append,
            ..Default::default()
        },
        CheckCommandDef {
            name: "only-deleted".to_string(),
            include: vec!["gone/**".to_string()],
            command: vec!["lint".to_string()],
            file_args: CheckFileArgs::Append,
            ..Default::default()
        },
        CheckCommandDef {
            name: "tsc".to_string(),
            include: vec!["**/*.ts".to_string()],
            command: vec!["tsc".to_string()],
            file_args: CheckFileArgs::None,
            ..Default::default()
        },
    ];
    let changed = vec![
        "a.ts".to_string(),
        "b.ts".to_string(),
        "gone/x.ts".to_string(),
    ];
    let deleted: BTreeSet<String> = ["a.ts".to_string(), "gone/x.ts".to_string()]
        .into_iter()
        .collect();
    let checks = generic_checks(&config, &changed, &deleted).unwrap();
    // Append: deleted files are dropped from the per-file args.
    let eslint = checks.iter().find(|c| c.name == "eslint").unwrap();
    assert_eq!(
        eslint.command,
        vec!["eslint".to_string(), "b.ts".to_string()]
    );
    // Append where every match is deleted: skipped entirely.
    assert!(!checks.iter().any(|c| c.name == "only-deleted"));
    // Whole-project check still triggers despite the deletion.
    assert!(checks.iter().any(|c| c.name == "tsc"));
}

#[test]
fn generic_checks_normalizes_dot_slash_globs() {
    use crate::config::v2::schema::{CheckCommandDef, CheckFileArgs};
    let mut config = NoMistakesConfig::default();
    config.checks.commands = vec![CheckCommandDef {
        name: "eslint".to_string(),
        include: vec!["./src/**/*.ts".to_string()],
        command: vec!["eslint".to_string()],
        file_args: CheckFileArgs::Append,
        ..Default::default()
    }];
    let checks = generic_checks(&config, &["src/foo.ts".to_string()], &BTreeSet::new()).unwrap();
    assert_eq!(checks.len(), 1);
    assert_eq!(
        checks[0].command,
        vec!["eslint".to_string(), "src/foo.ts".to_string()]
    );
}

#[test]
fn framework_present_detects_config_file() {
    let autodetect = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/impacted-checks/autodetect"),
    );
    let config = NoMistakesConfig::default();
    let visible = crate::codebase::ts_source::discover_visible_paths(&autodetect);
    // vitest.config.mts exists at root → Vitest is present without explicit config.
    assert!(framework_present(
        &autodetect,
        &config,
        TestFramework::Vitest,
        &visible,
    ));
    // No playwright.config / swift config → absent.
    assert!(!framework_present(
        &autodetect,
        &config,
        TestFramework::Playwright,
        &visible,
    ));
    assert!(!framework_present(
        &autodetect,
        &config,
        TestFramework::Swift,
        &visible,
    ));

    // testPlan full-suite triggers alone mark the framework present (no config
    // file needed): the basic fixture root has no vitest.config file.
    use crate::config::v2::schema::TestPlanIgnoredChangedTestsFramework;
    let mut plan_config = NoMistakesConfig::default();
    plan_config
        .test_plan
        .vitest
        .full_suite_triggers
        .ignore_changed_tests
        .push(TestPlanIgnoredChangedTestsFramework::Vitest);
    assert!(framework_present(
        &fixture(),
        &plan_config,
        TestFramework::Vitest,
        &[],
    ));
}

#[test]
fn framework_present_ignores_ignored_auto_configs_but_keeps_explicit_config_authority() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass3-visibility");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(fixture.path());
    let config = NoMistakesConfig::default();

    assert!(!framework_present(
        fixture.path(),
        &config,
        TestFramework::Vitest,
        &visible,
    ));
    assert!(!framework_present(
        fixture.path(),
        &config,
        TestFramework::Playwright,
        &visible,
    ));

    let mut explicit = config;
    explicit.tests.vitest.configs = Some(crate::config::v2::schema::StringOrList::One(
        "vitest.config.ts".to_string(),
    ));
    assert!(framework_present(
        fixture.path(),
        &explicit,
        TestFramework::Vitest,
        &visible,
    ));
}

#[test]
fn shell_quote_escapes_unsafe_tokens() {
    assert_eq!(shell_quote("vitest"), "vitest");
    assert_eq!(shell_quote("src/a b.ts"), "'src/a b.ts'");
    assert_eq!(shell_quote("it's"), "'it'\\''s'");
    assert_eq!(shell_quote(""), "''");
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

#[test]
fn multi_framework_fanout_matches_direct_framework_plans() {
    let fixture = fanout_fixture();
    let args = fanout_args(fixture.path());
    let report = generate_impacted_checks(&args).unwrap();

    let actual: BTreeSet<Vec<String>> = report
        .checks
        .iter()
        .filter(|check| check.kind == CheckKind::Test)
        .map(|check| check.command.clone())
        .collect();
    let mut direct = BTreeSet::new();
    for framework in [TestFramework::Vitest, TestFramework::Playwright] {
        let plan = crate::tests::plan::generate_plan(&plan_args_for(&args, Some(framework)))
            .expect("direct framework plan should succeed");
        for selected in plan.selected_tests {
            for target in selected.targets {
                let mut command = target.base_command;
                command.extend(target.runner_args);
                direct.insert(command);
            }
        }
    }

    assert_eq!(actual, direct);
    assert_eq!(actual.len(), 2, "{actual:?}");
    assert!(actual
        .iter()
        .flatten()
        .all(|part| !part.contains("ignored")));
}

#[test]
fn impacted_checks_napi_matches_the_cli_engine_for_multi_framework_fanout() {
    let fixture = fanout_fixture();
    let args = fanout_args(fixture.path());
    let cli_report = generate_impacted_checks(&args).unwrap();
    let options = serde_json::json!({
        "root": fixture.path().display().to_string(),
        "changedFiles": ["src/shared.ts"],
    });

    let napi_json = crate::napi_api::impacted_checks_json_impl(options.to_string()).unwrap();
    let napi_report: serde_json::Value = serde_json::from_str(&napi_json).unwrap();

    assert_eq!(napi_report, serde_json::to_value(cli_report).unwrap());
}

#[test]
fn impacted_fanout_prepares_and_builds_the_graph_once() {
    let generate = include_str!("generate.rs");
    let impacted_prepare = include_str!("generate/prepare.rs");
    let prepared = include_str!("../tests/prepared_plan.rs");
    let plan = include_str!("../tests/plan.rs");

    assert_eq!(
        impacted_prepare
            .matches("PreparedTestPlanInputs::prepare(&plan_args)")
            .count(),
        1
    );
    assert_eq!(impacted_prepare.matches("inputs.finish()?").count(), 1);
    assert_eq!(generate.matches("generate_plan_with_prepared(").count(), 1);
    assert_eq!(generate.matches("generate_plan(").count(), 0);
    for repeated_prepare in [
        "collect_changed_files(",
        "discover_visible_paths(",
        "load_v2_config(",
        "resolve_tsconfig(",
        "analyze_lockfile_changes(",
    ] {
        assert_eq!(
            generate.matches(repeated_prepare).count(),
            0,
            "fanout must not call {repeated_prepare}"
        );
    }

    assert_eq!(
        prepared.matches("VisiblePathSnapshot::new(&root)").count(),
        1
    );
    assert_eq!(prepared.matches("load_v2_config_from_visible(").count(), 1);
    assert_eq!(
        prepared.matches("resolve_tsconfig_from_visible(").count(),
        1
    );
    assert_eq!(
        prepared
            .matches("collect_changed_files(&args, &root)")
            .count(),
        1
    );
    assert_eq!(prepared.matches("analyze_lockfile_changes(").count(), 1);
    assert_eq!(
        prepared
            .matches("build_with_plan_files_prepared_config_and_facts(")
            .count(),
        1
    );
    assert_eq!(prepared.matches(".get_or_init(|| {").count(), 1);

    let public_wrapper = plan
        .split("pub fn generate_plan(args: &PlanArgs)")
        .nth(1)
        .and_then(|source| source.split("/// Generate a framework").next())
        .expect("public test-plan wrapper");
    assert_eq!(
        public_wrapper
            .matches("PreparedTestPlanRequest::prepare(args)")
            .count(),
        1
    );
    assert_eq!(
        public_wrapper
            .matches("generate_plan_with_prepared(")
            .count(),
        1
    );
}
