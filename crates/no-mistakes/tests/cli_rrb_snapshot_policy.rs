use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture;

type Case = (&'static str, std::path::PathBuf, std::path::PathBuf);

fn rule(label: &'static str, dir: &str, config: &str) -> Case {
    case(label, &format!("test-cases/rules/{dir}"), config)
}

fn unique(label: &'static str, scenario: &str) -> Case {
    let dir = format!("test-cases/codebase-analysis/unique-exports-{scenario}/fixture");
    case(label, &dir, ".no-mistakes.yml")
}

#[test]
fn coverage_producer_assertion_owner_uncertainty_json_matches_snapshot() {
    let dir = "coverage-producer-assertion-owner-uncertainty/fixture";
    assert_saved_fixture(
        "coverage_producer_assertion_owner_uncertainty",
        &[
            rule("dispatch", dir, "dispatch.yml"),
            rule("depth", dir, "depth.yml"),
            rule("nodes", dir, "nodes.yml"),
            rule("mixed", dir, "mixed.yml"),
            rule("both", dir, "both.yml"),
            rule("custom", dir, "custom.yml"),
            rule("advise", dir, "advise.yml"),
            rule("source", dir, "source.yml"),
            rule("disabled", dir, "disabled.yml"),
        ],
    );
}

#[test]
fn coverage_producer_assertion_owner_builtins_json_matches_snapshot() {
    let dir = "coverage-producer-assertion-owner-builtins/fixture";
    let names = [
        "pass",
        "no-owner",
        "callback-named",
        "callback-inline",
        "callback-storage",
        "callback-stored",
        "callback-unresolved",
        "third-party",
        "third-party-catch",
        "push-sink",
        "record-sink",
        "replace-string",
        "replace-inline",
        "replace-unresolved",
        "many",
        "globals",
        "imported-callback",
        "continuations",
        "review-parent",
        "review-parent-ignore",
        "review-storage",
        "review-context",
        "review-timer",
        "review-array",
        "review-array-ignore",
        "json-reviver",
        "member-callback",
        "namespace-all",
        "namespace-default",
        "namespace-named",
        "namespace-export",
        "namespace-shadowed",
        "map-get",
        "array-callback",
        "namespace-nested",
        "namespace-ambiguous",
        "namespace-cycle",
        "namespace-bare",
        "namespace-value",
        "builtin-name-only",
        "external-nested",
        "disabled",
    ];
    let mut cases: Vec<Case> = names
        .iter()
        .map(|name| rule(name, dir, &format!("{name}.yml")))
        .collect();
    cases.push(rule(
        "without-storage",
        "coverage-producer-assertion-owner-builtins-no-storage/fixture",
        "check.yml",
    ));
    cases.push(rule(
        "storage-closure",
        "coverage-producer-assertion-owner-storage-closure/fixture",
        "check.yml",
    ));
    assert_saved_fixture("coverage_producer_assertion_owner_builtins", &cases);
}

#[test]
fn coverage_producer_assertion_owner_external_stars_json_matches_snapshot() {
    let dir = "coverage-producer-assertion-owner-external-stars/fixture";
    assert_saved_fixture(
        "coverage_producer_assertion_owner_external_stars",
        &[
            rule("enabled", dir, ".no-mistakes.yml"),
            rule("disabled", dir, "disabled.yml"),
        ],
    );
}

#[test]
fn coverage_producer_assertion_owner_json_matches_snapshot() {
    let dir = "coverage-producer-assertion-owner/fixture";
    assert_saved_fixture(
        "coverage_producer_assertion_owner",
        &[
            rule("default", dir, ".no-mistakes.yml"),
            rule("advise", dir, "advise.yml"),
            rule("ignore", dir, "ignore.yml"),
            rule("disabled", dir, "disabled.yml"),
            rule("filtered", dir, "filtered.yml"),
            rule("suppression", dir, "suppression.yml"),
            rule("mixed", dir, "mixed.yml"),
            rule("nodes", dir, "nodes.yml"),
            rule("switches", dir, "switches.yml"),
            rule("custom", dir, "custom.yml"),
            rule("applications", dir, "applications.yml"),
            rule("depth", dir, "depth.yml"),
            rule("generators", dir, "generators-report.yml"),
            rule("import_forms", dir, "import-forms-report.yml"),
            rule("parse_error", dir, "parse-error.yml"),
            rule("ambiguous_callback", dir, "ambiguous-callback.yml"),
            rule("missing_file", dir, "missing-file.yml"),
            rule("typo", dir, "typo.yml"),
        ],
    );
}

#[test]
fn coverage_producer_assertion_owner_fs_json_matches_snapshot() {
    assert_saved_fixture(
        "coverage_producer_assertion_owner_fs",
        &[rule(
            "fs_io",
            "coverage-producer-assertion-owner-fs/fixture",
            ".no-mistakes.yml",
        )],
    );
}

#[test]
fn declared_payload_compatibility_json_matches_snapshot() {
    let dir = "fixtures/rules/declared-payload-compatibility";
    assert_saved_fixture(
        "declared_payload_compatibility",
        &[
            case("fail", &format!("{dir}/fail"), ".no-mistakes.yml"),
            case("pass", &format!("{dir}/pass"), ".no-mistakes.yml"),
        ],
    );
}

#[test]
fn doc_consistency_json_matches_snapshot() {
    let dir = "doc-consistency/fixture";
    assert_saved_fixture(
        "doc_consistency",
        &[
            rule("fail", &format!("{dir}/fail"), ".no-mistakes.yml"),
            rule("pass", &format!("{dir}/pass"), ".no-mistakes.yml"),
            rule("patterns", &format!("{dir}/patterns"), ".no-mistakes.yml"),
        ],
    );
}

#[test]
fn finite_set_consistency_json_matches_snapshot() {
    assert_saved_fixture(
        "finite_set_consistency",
        &[rule(
            "glob_coverage",
            "finite-set-consistency/fixture",
            "../config/finite_set_consistency_glob_coverage_appears_in_cli_json.yml",
        )],
    );
}

#[test]
fn lockfile_allowlist_json_matches_snapshot() {
    let dir = "lockfile-allowlist/fixture";
    assert_saved_fixture(
        "lockfile_allowlist",
        &[
            rule("fail", &format!("{dir}/fail"), ".no-mistakes.yml"),
            rule("pass", &format!("{dir}/pass"), ".no-mistakes.yml"),
        ],
    );
}

#[test]
fn unique_exports_json_matches_snapshot() {
    assert_saved_fixture(
        "unique_exports",
        &[
            unique("basic", "basic"),
            unique("strict", "strict"),
            unique("edge_cases", "edge-cases"),
            unique("type_value_strict", "type-value-strict"),
            unique("path_filters", "path-filters"),
            unique("config_disabled", "config-disabled"),
        ],
    );
}
