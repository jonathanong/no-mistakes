use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture;

type Case = (&'static str, std::path::PathBuf, std::path::PathBuf);

fn c(label: &'static str, dir: &str, config: &str) -> Case {
    case(label, &format!("test-cases/rules/{dir}"), config)
}

fn correspondence(label: &'static str, scenario: &str) -> Case {
    let dir = format!("vitest-test-correspondence/fixture/{scenario}");
    c(label, &dir, ".no-mistakes.yml")
}

#[test]
fn integration_test_no_mocks_json_matches_snapshot() {
    assert_saved_fixture(
        "integration_test_no_mocks",
        &[c(
            "mocked",
            "integration-test-no-mocks/fixture",
            ".no-mistakes.yml",
        )],
    );
}

#[test]
fn test_email_domain_policy_json_matches_snapshot() {
    assert_saved_fixture(
        "test_email_domain_policy",
        &[c(
            "fail",
            "test-email-domain-policy/fixture",
            ".no-mistakes.yml",
        )],
    );
}

#[test]
fn test_no_dependency_pins_json_matches_snapshot() {
    assert_saved_fixture(
        "test_no_dependency_pins",
        &[
            c(
                "fail",
                "test-no-dependency-pins/fixture/fail",
                ".no-mistakes.yml",
            ),
            c(
                "pass",
                "test-no-dependency-pins/fixture/pass",
                ".no-mistakes.yml",
            ),
        ],
    );
}

#[test]
fn test_no_unmocked_dynamic_imports_json_matches_snapshot() {
    let dir = "test-no-unmocked-dynamic-imports/fixture/effort-low";
    assert_saved_fixture(
        "test_no_unmocked_dynamic_imports",
        &[
            c("default_max", dir, ".no-mistakes-default-max.yml"),
            c("report", dir, ".no-mistakes-report.yml"),
            c("advise", dir, ".no-mistakes-advise.yml"),
            c("ignore", dir, ".no-mistakes-ignore.yml"),
        ],
    );
}

#[test]
fn vitest_ci_path_coverage_json_matches_snapshot() {
    let config = "../config/vitest_ci_path_coverage_appears_in_cli_json.yml";
    assert_saved_fixture(
        "vitest_ci_path_coverage",
        &[
            c("fail", "vitest-ci-path-coverage/fixture", config),
            c("pass", "vitest-ci-path-coverage/pass", config),
        ],
    );
}

#[test]
fn vitest_project_mapping_json_matches_snapshot() {
    let config =
        "../config/generic_filesystem_rules_run_through_public_dispatch-vitest-project-mapping.yml";
    assert_saved_fixture(
        "vitest_project_mapping",
        &[
            c("unmapped", "vitest-project-mapping/fixture", config),
            c(
                "dynamic_config",
                "vitest-project-mapping/dynamic-config",
                ".no-mistakes.yml",
            ),
        ],
    );
}

#[test]
fn vitest_test_correspondence_json_matches_snapshot() {
    assert_saved_fixture(
        "vitest_test_correspondence",
        &[
            correspondence("fail", "fail"),
            correspondence("pass", "pass"),
            correspondence("source_to_test_fail", "source-to-test-fail"),
            correspondence("source_to_test_pass", "source-to-test-pass"),
            correspondence("test_to_source_fail", "test-to-source-fail"),
            correspondence("test_to_source_pass", "test-to-source-pass"),
            correspondence("stem_suffix", "stem-suffix-diagnostics"),
            correspondence("tsx_missing_test", "tsx-source-missing-test"),
            correspondence("declarations_pass", "declarations-pass"),
        ],
    );
}
