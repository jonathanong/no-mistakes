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
fn query_reached_per_item_call_reachability_json_matches_snapshot() {
    assert_saved_fixture(
        "query_reached_per_item_call_reachability",
        &[
            rule(
                "call_reachability",
                "query-reached-per-item/call-reachability-fixture",
                ".no-mistakes.yml",
            ),
            rule(
                "imported_stages",
                "query-reached-per-item/imported-stages-fixture",
                ".no-mistakes.yml",
            ),
            rule(
                "namespace_member",
                "query-reached-per-item/namespace-member-stages-fixture",
                ".no-mistakes.yml",
            ),
            rule(
                "namespace_cycle",
                "query-reached-per-item/namespace-cycle-stages-fixture",
                ".no-mistakes.yml",
            ),
            rule(
                "namespace_unknown",
                "query-reached-per-item/namespace-unknown-stages-fixture",
                ".no-mistakes.yml",
            ),
            rule(
                "guarded_workspace",
                "query-reached-per-item/guarded-workspace-stages-fixture",
                ".no-mistakes.yml",
            ),
            rule(
                "guarded_workspace_typed",
                "query-reached-per-item/guarded-workspace-stages-fixture",
                "typed.yml",
            ),
            rule(
                "guard_shapes",
                "query-reached-per-item/guard-shapes-fixture",
                ".no-mistakes.yml",
            ),
            rule(
                "contextual_stages",
                "query-reached-per-item/typed-contextual-stages-fixture",
                ".no-mistakes.yml",
            ),
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

#[test]
fn query_reached_per_item_capped_cursor_check_json_snapshot() {
    let dir = "query-reached-per-item/capped-cursor";
    assert_saved_fixture(
        "query_reached_per_item_capped_cursor",
        &[
            rule("default", dir, ".no-mistakes.yml"),
            rule("off", dir, "off.yml"),
            rule("on", dir, "on.yml"),
        ],
    );
}

#[test]
fn query_reached_per_item_counter_check_json_snapshot() {
    let dir = "query-reached-per-item/bounded-shapes/counter";
    assert_saved_fixture(
        "query_reached_per_item_counter",
        &[
            rule("enabled", dir, ".no-mistakes.yml"),
            rule("disabled", dir, "off.yml"),
        ],
    );
}

#[test]
fn query_reached_per_item_tuple_enum_check_json_snapshot() {
    let dir = "query-reached-per-item/bounded-shapes/tuple-enum";
    assert_saved_fixture(
        "query_reached_per_item_tuple_enum",
        &[
            rule("enabled", dir, ".no-mistakes.yml"),
            rule("disabled", dir, "off.yml"),
        ],
    );
}

#[test]
fn query_reached_per_item_chunk_check_json_snapshot() {
    let dir = "query-reached-per-item/bounded-shapes/chunk";
    assert_saved_fixture(
        "query_reached_per_item_chunk",
        &[
            rule("enabled", dir, ".no-mistakes.yml"),
            rule("disabled", dir, "off.yml"),
        ],
    );
}

#[test]
fn query_reached_per_item_keyset_check_json_snapshot() {
    let dir = "query-reached-per-item/bounded-shapes/keyset";
    assert_saved_fixture(
        "query_reached_per_item_keyset",
        &[
            rule("enabled", dir, ".no-mistakes.yml"),
            rule("disabled", dir, "off.yml"),
        ],
    );
}

#[test]
fn query_reached_per_item_closed_keys_check_json_snapshot() {
    let dir = "query-reached-per-item/bounded-shapes/closed-keys";
    assert_saved_fixture(
        "query_reached_per_item_closed_keys",
        &[
            rule("enabled", dir, ".no-mistakes.yml"),
            rule("disabled", dir, "off.yml"),
        ],
    );
}

#[test]
fn query_reached_per_item_distinct_calls_check_json_snapshot() {
    let dir = "query-reached-per-item/bounded-shapes/distinct-calls";
    assert_saved_fixture(
        "query_reached_per_item_distinct_calls",
        &[
            rule("enabled", dir, ".no-mistakes.yml"),
            rule("disabled", dir, "off.yml"),
        ],
    );
}

#[test]
fn query_reached_per_item_dependent_chain_check_json_snapshot() {
    let dir = "query-reached-per-item/bounded-shapes/dependent-chain";
    assert_saved_fixture(
        "query_reached_per_item_dependent_chain",
        &[
            rule("enabled", dir, ".no-mistakes.yml"),
            rule("disabled", dir, "off.yml"),
        ],
    );
}

#[test]
fn query_reached_per_item_allow_function_check_json_snapshot() {
    let dir = "query-reached-per-item/allow-function";
    assert_saved_fixture(
        "query_reached_per_item_allow_function",
        &[
            rule("unallowed", dir, "no-allow.yml"),
            rule("function", dir, ".no-mistakes.yml"),
            rule("file_only", dir, "file-only.yml"),
            rule("unused", dir, "hygiene.yml"),
            rule("disabled", dir, "off.yml"),
        ],
    );
}
