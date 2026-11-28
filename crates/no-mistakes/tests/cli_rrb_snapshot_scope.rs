use super::check_json_case::case;
use super::check_json_snapshot::{assert_saved_fixture, assert_saved_fixture_report};

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
            production("suppression_forms", "suppression-forms"),
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
fn query_reached_per_item_stage_import_index_check_json_snapshot() {
    let dir = "query-reached-per-item/stage-import-index-workload";
    assert_saved_fixture(
        "query_reached_per_item_stage_import_index",
        &[
            rule("many_origins", dir, ".no-mistakes.yml"),
            rule("no_consumers", dir, "off.yml"),
        ],
    );
}

#[test]
fn query_reached_per_item_scoped_stage_check_json_snapshot() {
    let dir = "query-reached-per-item/stage-import-index-workload";
    assert_saved_fixture(
        "query_reached_per_item_scoped_stage",
        &[
            rule("caller_only", dir, "caller-only.yml"),
            rule("noise_only", dir, "noise-only.yml"),
            rule("noise_only_allow", dir, "noise-only-allow.yml"),
        ],
    );
}

#[test]
fn query_reached_per_item_checks_scoped_occurrences_before_effect_sinks() {
    let all = check_stage_scope(".no-mistakes.yml");
    let caller = check_stage_scope("caller-only.yml");
    let noise = check_stage_scope("noise-only.yml");
    let off = check_stage_scope("off.yml");

    assert_eq!(caller.status.code(), Some(1));
    assert_eq!(caller.stdout, all.stdout);
    assert_eq!(noise.status.code(), Some(0));
    assert_eq!(noise.stdout, off.stdout);
    assert_eq!(query_projection_work(&caller, "candidate_positions"), 48);
    assert_eq!(query_projection_work(&caller, "candidate_sites"), 48);
    assert!(query_projection_work(&caller, "sink_sites") > 48);
    for metric in ["candidate_positions", "candidate_sites", "sink_sites"] {
        assert_eq!(query_projection_work(&noise, metric), 0, "{metric}");
    }
    assert!(String::from_utf8_lossy(&noise.stderr)
        .lines()
        .any(|line| line == "[work] graph.builds: 1"));
}

fn check_stage_scope(config: &str) -> std::process::Output {
    let (_, root, config) = rule(
        "scope",
        "query-reached-per-item/stage-import-index-workload",
        config,
    );
    std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .current_dir(&root)
        .env_remove("RAYON_NUM_THREADS")
        .args(["--lock-timeout", "0", "--jobs", "2", "check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .args(["--format", "json", "--verbose-timings"])
        .output()
        .unwrap()
}

fn query_projection_work(output: &std::process::Output, metric: &str) -> u64 {
    let prefix = format!("[work] query_reached_per_item.{metric}: ");
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("missing work counter: {metric}"))
        .parse()
        .unwrap()
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
                "disable_file",
                "required-companion-imports/disable-file",
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
                "disable_file",
                &format!("{dir}/disable-file"),
                ".no-mistakes.yml",
            ),
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
fn invalid_utf8_required_doc_section_json_matches_snapshot() {
    assert_saved_fixture(
        "invalid_utf8_required_doc_section",
        &[rule(
            "missing_heading",
            "required-doc-section/fixture/invalid-utf8",
            ".no-mistakes.yml",
        )],
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
            rule(
                "config_disable_line",
                "required-entrypoint-reachability/suppression/disable-line",
                ".no-mistakes.yml",
            ),
            rule(
                "config_disable_next_line",
                "required-entrypoint-reachability/suppression/disable-next-line",
                ".no-mistakes.yml",
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
                "escape_disable_file",
                &format!("{dir}/escape-disable-file"),
                ".no-mistakes.yml",
            ),
            rule(
                "escape_disable_line",
                &format!("{dir}/escape-disable-line"),
                ".no-mistakes.yml",
            ),
            rule(
                "escape_disable_next_line",
                &format!("{dir}/escape-disable-next-line"),
                ".no-mistakes.yml",
            ),
            rule(
                "directory_suppression",
                &format!("{dir}/directory-suppression"),
                ".no-mistakes.yml",
            ),
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
fn query_reached_per_item_language_call_policy() {
    let value = assert_saved_fixture_report(
        "query_reached_per_item_language_call_policy",
        &[rule(
            "languages",
            "query-reached-per-item/language-call-policy",
            "policy.yml",
        )],
        &[],
        &[],
    );
    let report = &value["languages"]["report"];
    let files = [
        "python/worker.py",
        "go/worker.go",
        "rust/worker.rs",
        "ruby/worker.rb",
        "php/worker.php",
        "java/Worker.java",
        "kotlin/Worker.kt",
        "elixir/worker.ex",
        "dart/worker.dart",
        "swift/Worker.swift",
        "csharp/Worker.cs",
    ];
    for file in files {
        assert!(
            report["rules"].as_array().is_some_and(|findings| {
                findings.iter().any(|finding| {
                    finding["file"] == file
                        && finding["message"].as_str().unwrap().contains("read_row")
                })
            }),
            "{file} proven finding missing: {report}"
        );
        assert!(
            report["advisories"].as_array().is_some_and(|findings| {
                findings.iter().any(|finding| {
                    finding["file"] == file && finding["causes"][0]["kind"] == "unresolved-dispatch"
                })
            }),
            "{file} unproven advisory missing: {report}"
        );
    }
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
