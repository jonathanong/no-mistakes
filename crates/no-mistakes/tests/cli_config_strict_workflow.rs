use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/strict-runner-infra/fixture")
        .canonicalize()
        .unwrap()
}

fn invoke(root: &Path, config: &str, command: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--jobs", "1"])
        .args(command)
        .arg("--root")
        .arg(root)
        .args(["--config", config, "--format", "json"])
        .output()
        .unwrap()
}

#[test]
fn invalid_runner_infra_and_changes_configuration_exits_two_at_load() {
    let root = fixture();
    for (config, field) in [
        ("4427.yml", "tests.impact.registries[0]"),
        ("4428.yml", "tests.impact.alwaysIncludeTests[0]"),
        (
            "4430.yml",
            "testPlan.vitest.environments.pre-push.limit.percent",
        ),
        ("4430-range.yml", "101"),
        (
            "4430-group.yml",
            "testPlan.vitest.environments.pre-push.groups[0].limit.percent",
        ),
        (
            "4435.yml",
            "testPlan.vitest.environments.pre-push.include[0]",
        ),
        (
            "4438.yml",
            "testPlan.vitest.environments.pre-push.exclude[0]",
        ),
        (
            "4439.yml",
            "testPlan.vitest.fullSuiteTriggers.projects.missing",
        ),
        (
            "4440.yml",
            "testPlan.vitest.fullSuiteTriggers.projects.example[0]",
        ),
        ("4456.yml", "infra.terrafom"),
        ("4457.yml", "infra.terraform.moduleRoot"),
        ("4458.yml", "infra.terraform.test.testGlob"),
        ("4459.yml", "tests.vitset"),
        ("4461.yml", "tests.vitest.projects.example.inculde"),
        ("4461-jest.yml", "tests.jest.projects.example.inculde"),
        (
            "4461-playwright.yml",
            "tests.playwright.projects.example.inculde",
        ),
        ("4462.yml", "tests.dotnet.projects.example.inculde"),
        ("4463.yml", "tests.playwright.selectors.test_ids"),
        ("4465.yml", "tests.playwright.coverage.route"),
        ("4466.yml", "tests.playwright.apps.example.selectorRoot"),
        ("4467.yml", "tests.impact.registry"),
        ("4469.yml", "testPlan.vitset"),
        ("4470.yml", "testPlan.vitest.environment"),
        ("4471.yml", "testPlan.vitest.environments.pre-push.limti"),
        (
            "4473.yml",
            "testPlan.vitest.environments.pre-push.groups[0].tyep",
        ),
        (
            "4474.yml",
            "testPlan.vitest.environments.pre-push.limit.file",
        ),
        ("4475.yml", "testPlan.vitest.fullSuiteTriggers.trigger"),
        ("4479.yml", "tests.playwright.configsTypo"),
        (
            "4480-include.yml",
            "tests.playwright.routeCoverageSources[0].include",
        ),
        (
            "4480-routes.yml",
            "tests.playwright.routeCoverageSources[0].routes",
        ),
        (
            "4480-helpers.yml",
            "tests.playwright.routeCoverageSources[0].helpers",
        ),
        (
            "4481.yml",
            "tests.playwright.routeCoverageSources[0].include[0]",
        ),
        (
            "4481-app.yml",
            "tests.playwright.apps.example.routeCoverageSources[0].include[0]",
        ),
        ("4483.yml", "infra.terraform.test.testGlobs[0]"),
        ("4484.yml", "infra.terraform.test.match"),
        ("4484-shared-module.yml", "infra.terraform.test.testRoot"),
        ("4486.yml", "changes.categories.example.paths[0]"),
        ("4487.yml", "changes.categories.example.exclude[0]"),
        ("4488.yml", "changes.categories.example.tests[0]"),
        ("4490.yml", "changes.categories.example.rules[0].paths"),
        (
            "4490-glob.yml",
            "changes.categories.example.rules[0].paths[0]",
        ),
        ("4491.yml", "changes.categories.example.id.fromPath"),
        ("4491-double.yml", "changes.categories.example.id.fromPath"),
        ("4492.yml", "changes.categories.example.match"),
    ] {
        let out = invoke(&root, config, &["check"]);
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert_eq!(out.status.code(), Some(2), "{config}: {stderr}");
        assert!(
            out.stdout.is_empty(),
            "{config}: published a partial report"
        );
        assert!(
            stderr.contains(config) && stderr.contains(field),
            "{stderr}"
        );
    }

    let out = invoke(&root, "valid.yml", &["check"]);
    assert_eq!(out.status.code(), Some(0), "{:?}", out.stderr);
    assert!(serde_json::from_slice::<serde_json::Value>(&out.stdout).is_ok());
}

#[test]
fn invalid_test_plan_limit_uses_the_shared_config_diagnostic() {
    let root = fixture();
    let out = invoke(
        &root,
        "4430.yml",
        &["tests", "plan", "vitest", "--changed-file", "example.ts"],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).unwrap().replace('\\', "/");
    let stderr = stderr.replace(&root.to_string_lossy().replace('\\', "/"), "<root>");
    insta::assert_snapshot!("invalid_test_plan_limit", stderr.trim());
}
