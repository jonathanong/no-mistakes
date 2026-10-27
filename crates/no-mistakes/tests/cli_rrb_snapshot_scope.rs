use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture;

type Case = (&'static str, std::path::PathBuf, std::path::PathBuf);

fn rule(label: &'static str, dir: &str, config: &str) -> Case {
    case(label, &format!("test-cases/rules/{dir}"), config)
}

fn production(label: &'static str, scenario: &str) -> Case {
    let dir = format!("production-dependency-declarations/{scenario}");
    rule(label, &dir, "../configs/workspace-packages.yml")
}

#[test]
fn production_dependency_declarations_json_matches_snapshot() {
    assert_saved_fixture(
        "production_dependency_declarations",
        &[
            production("undeclared", "undeclared-import"),
            production("dev_dependency", "third-party-devdependency"),
            production("dev_only_workspace", "dev-only-production-import"),
            production("relative_closure", "relative-import-closure"),
            production("type_import", "import-type-exempt"),
            production("suppressed", "suppression-honored"),
            production("test_only_importer", "test-only-importer"),
            production("computed_import", "computed-import"),
        ],
    );
}

#[test]
fn query_reached_per_item_json_matches_snapshot() {
    let dir = "query-reached-per-item/fixture";
    assert_saved_fixture(
        "query_reached_per_item",
        &[
            rule("default", dir, ".no-mistakes.yml"),
            rule("category", dir, "category.yml"),
            rule("allow", dir, "allow.yml"),
            rule("disabled", dir, "disabled.yml"),
        ],
    );
}

#[test]
fn required_companion_imports_json_matches_snapshot() {
    assert_saved_fixture(
        "required_companion_imports",
        &[
            rule(
                "components",
                "required-companion-imports/fixture",
                "../config/generic_filesystem_rules_run_through_public_dispatch-required-companion-imports.yml",
            ),
            rule(
                "prepared_scope",
                "required-companion-imports/prepared-scope",
                ".no-mistakes.yml",
            ),
            rule(
                "prepared_shared",
                "required-companion-imports/prepared-shared",
                ".no-mistakes.yml",
            ),
            rule(
                "prepared_external",
                "required-companion-imports/prepared-external/root",
                ".no-mistakes.yml",
            ),
            rule(
                "repository_scope_abort",
                "required-companion-imports/repository-scope-abort",
                ".no-mistakes.yml",
            ),
        ],
    );
}

#[test]
fn required_doc_section_json_matches_snapshot() {
    let dir = "required-doc-section/fixture";
    assert_saved_fixture(
        "required_doc_section",
        &[
            rule("fail", &format!("{dir}/fail"), ".no-mistakes.yml"),
            rule("pass", &format!("{dir}/pass"), ".no-mistakes.yml"),
            rule(
                "deeper_heading",
                &format!("{dir}/fail-deeper-heading"),
                ".no-mistakes.yml",
            ),
            rule(
                "fenced_heading",
                &format!("{dir}/fail-fenced-heading"),
                ".no-mistakes.yml",
            ),
        ],
    );
}

#[test]
fn required_entrypoint_reachability_json_matches_snapshot() {
    assert_saved_fixture(
        "required_entrypoint_reachability",
        &[
            rule(
                "sources",
                "required-entrypoint-reachability/fixture",
                "../configs/sources.yml",
            ),
            rule(
                "suppression",
                "required-entrypoint-reachability/fixture",
                "suppression.no-mistakes.yml",
            ),
            case(
                "filesystem_skip",
                "test-cases/check-runner/required-reachability-ignores-filesystem-skip/fixture",
                ".no-mistakes.yml",
            ),
        ],
    );
}

#[test]
fn required_local_docs_json_matches_snapshot() {
    let dir = "required-local-docs/fixture";
    assert_saved_fixture(
        "required_local_docs",
        &[
            rule("fail", &format!("{dir}/fail"), ".no-mistakes.yml"),
            rule("pass", &format!("{dir}/pass"), ".no-mistakes.yml"),
            rule("escape", &format!("{dir}/escape"), ".no-mistakes.yml"),
            rule(
                "parent_segment",
                &format!("{dir}/parent-segment"),
                ".no-mistakes.yml",
            ),
        ],
    );
}
