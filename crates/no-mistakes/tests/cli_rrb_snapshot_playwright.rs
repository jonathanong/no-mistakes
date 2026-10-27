use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture;

type Case = (&'static str, std::path::PathBuf, std::path::PathBuf);

fn c(label: &'static str, dir: &str, config: &str) -> Case {
    case(label, &format!("test-cases/check-runner/{dir}"), config)
}

#[test]
fn playwright_coverage_json_matches_snapshot() {
    let dir = "playwright-coverage/fixture/effort-low";
    assert_saved_fixture(
        "playwright_coverage",
        &[
            c("default_max", dir, ".no-mistakes-default-max.yml"),
            c("report", dir, ".no-mistakes-report.yml"),
            c("advise", dir, ".no-mistakes-advise.yml"),
            c("ignore", dir, ".no-mistakes-ignore.yml"),
        ],
    );
}

#[test]
fn playwright_prefer_test_id_locators_json_matches_snapshot() {
    assert_saved_fixture(
        "playwright_prefer_test_id_locators",
        &[c(
            "fail",
            "playwright-prefer-test-id-locators/fixture",
            ".no-mistakes.yml",
        )],
    );
}

#[test]
fn playwright_unique_html_ids_json_matches_snapshot() {
    let dir = "playwright-unique-html-ids/fixture/effort-low";
    assert_saved_fixture(
        "playwright_unique_html_ids",
        &[
            c("default_max", dir, ".no-mistakes-default-max.yml"),
            c("report", dir, ".no-mistakes-report.yml"),
            c("advise", dir, ".no-mistakes-advise.yml"),
            c("ignore", dir, ".no-mistakes-ignore.yml"),
            c("mixed", dir, ".no-mistakes-mixed.yml"),
        ],
    );
}

#[test]
fn playwright_unique_test_ids_json_matches_snapshot() {
    let low = "playwright-unique-test-ids/fixture/effort-low";
    let owned = "playwright-unique-test-ids/fixture/html-id-ownership";
    assert_saved_fixture(
        "playwright_unique_test_ids",
        &[
            c(
                "duplicate",
                "playwright-unique-test-ids/fixture",
                ".no-mistakes.yml",
            ),
            c("default_max", low, ".no-mistakes-default-max.yml"),
            c("report", low, ".no-mistakes-report.yml"),
            c("advise", low, ".no-mistakes-advise.yml"),
            c("ignore", low, ".no-mistakes-ignore.yml"),
            c("coverage_report", owned, "coverage-report.yml"),
            c("configured_report", owned, "configured-report.yml"),
            c("component_report", owned, "component-report.yml"),
            c("dedicated_html_report", owned, "dedicated-html-report.yml"),
        ],
    );
}
