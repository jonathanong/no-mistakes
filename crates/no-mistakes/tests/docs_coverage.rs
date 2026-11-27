#[path = "support/docs_audit.rs"]
mod docs_audit;
#[path = "support/docs_include_fragment_regressions.rs"]
mod docs_include_fragment_regressions;
#[path = "support/docs_lint_suppression_regressions.rs"]
mod docs_lint_suppression_regressions;
#[path = "common/saved_fixture.rs"]
mod saved_fixture;

#[path = "support/docs_node_runtime_exports.rs"]
mod docs_node_runtime_exports;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[path = "support/docs_coverage_cli_helpers.rs"]
mod cli_docs_helpers;
#[path = "support/docs_coverage_rule_helpers.rs"]
mod rule_docs_helpers;
#[path = "support/docs_rule_inventory.rs"]
mod rule_inventory;
#[path = "support/docs_rule_inventory_regressions.rs"]
mod rule_inventory_regressions;
use rule_docs_helpers::{read, repo_root, rule_ids_from_source};
#[path = "support/docs_review_regressions.rs"]
mod docs_review_regressions;
use cli_docs_helpers::{
    enum_block, enum_variants, kebab_case, reachable_cli_pages, rust_sources, subcommand_enums,
};

#[path = "support/docs_coverage_commands.rs"]
mod docs_coverage_commands;
#[path = "support/docs_coverage_csp_allowed_origins.rs"]
mod docs_coverage_csp_allowed_origins;
#[path = "support/docs_coverage_rules.rs"]
mod docs_coverage_rules;
#[path = "support/docs_coverage_test_reliability.rs"]
mod docs_coverage_test_reliability;
#[path = "support/docs_rule_inventory_checks.rs"]
mod docs_rule_inventory_checks;

#[test]
fn review_found_doc_regressions_stay_fixed() {
    let root = repo_root();
    let read_root = |path: &str| read(&root.join(path));

    let configuration = read_root("docs/configuration/README.md");
    assert!(!configuration.contains("legacy.md"));

    let node_api = read_root("docs/node-api.md");
    assert!(node_api.contains("(async () => {"));
    assert!(node_api.contains("playwright check\\|edges\\|related\\|tests"));
    assert!(node_api.contains("queues edges\\|related\\|check"));
    assert!(node_api.contains("server routes\\|edges\\|related\\|contracts"));
    assert!(node_api.contains("react analyze\\|check"));

    let eslint_plugin = read_root("docs/eslint-plugin.md");
    assert!(eslint_plugin.contains(r#"("named" | "default")[]"#));

    for rule_doc in [
        "docs/rules/forbidden-dependencies.md",
        "docs/rules/no-empty-or-comments-only-files.md",
    ] {
        let body = read_root(rule_doc);
        assert!(body.contains("Compliant example:"), "{rule_doc}");
        assert!(body.contains("Suppression caveat:"), "{rule_doc}");
    }

    let limits =
        read_root("plugins/no-mistakes/skills/no-mistakes/references/limits-and-fallbacks.md");
    assert!(limits.contains(r#"spawn("scripts/seed.mts", [])"#));
    assert!(limits.contains("spawn(scriptName, [])"));
    assert!(limits.contains(r#"page.locator('[data-testid="submit"]').click()"#));
    assert!(limits.contains(r#"page.locator(`[data-testid="${id}"]`).click()"#));
    assert!(limits.contains("--relationship python"));
    assert!(!limits.contains(
        "Other file types (Go, Rust, Python, CSS, JSON, Markdown source files) are not walked."
    ));

    let feature_parity = read_root("docs/feature-parity.md");
    for needle in [
        "Python",
        "Django",
        "Celery",
        "Go",
        "Asynq",
        "Kafka",
        "Rust",
        "Ruby on Rails",
        "PHP",
        "Sidekiq",
        "Counterexample:",
        "native fallback only",
        "`Cargo.lock`",
        "`Gemfile.lock`",
        "`composer.lock`",
    ] {
        assert!(
            feature_parity.contains(needle),
            "docs/feature-parity.md must document `{needle}`"
        );
    }
    let docs_index = read_root("docs/README.md");
    assert!(
        docs_index.contains("(feature-parity.md)"),
        "docs/README.md must link feature-parity.md"
    );
    assert!(docs_index.contains("(why.md)"));

    let why = read_root("docs/why.md");
    assert!(why.contains("Three jobs"));
    assert!(why.contains("Mistakes this prevents"));
}

#[test]
fn shell_compatibility_rules_are_registered_and_documented() {
    let root = repo_root();
    let rules = rule_ids_from_source(&root);
    for rule in [
        no_mistakes::codebase::rules::BASH_EMPTY_ARRAY_NOUNSET,
        no_mistakes::codebase::rules::ZSH_READONLY_PARAMETER,
    ] {
        assert!(rules.contains(rule));
        for doc in [
            "docs/cli/check.md",
            "docs/node-api.md",
            "docs/configuration/rules.md",
        ] {
            assert!(read(&root.join(doc)).contains(rule), "{doc} omits {rule}");
        }
    }
}

#[test]
fn github_actions_max_inline_script_lines_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::GITHUB_ACTIONS_MAX_INLINE_SCRIPT_LINES;
    assert_eq!(rule, "github-actions-max-inline-script-lines");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    assert!(repo_root().join(format!("docs/rules/{rule}.md")).is_file());
}

#[test]
fn no_single_enqueue_in_loop_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::NO_SINGLE_ENQUEUE_IN_LOOP;
    assert_eq!(rule, "no-single-enqueue-in-loop");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    assert!(repo_root().join(format!("docs/rules/{rule}.md")).is_file());
    assert!(read(&repo_root().join("docs/configuration/rules.md")).contains(rule));
}

#[test]
fn github_actions_cache_key_policy_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::GITHUB_ACTIONS_CACHE_KEY_POLICY;
    assert_eq!(rule, "github-actions-cache-key-policy");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    let root = repo_root();
    let doc = read(&root.join(format!("docs/rules/{rule}.md")));
    assert!(doc.contains("forbiddenPaths"));
    assert!(doc.contains("forbiddenPathsMessage"));
    let config_doc = read(&root.join("docs/configuration/rules.md"));
    assert!(config_doc.contains(rule));
    assert!(config_doc.contains("forbiddenPaths"));
    let node_doc = read(&root.join("docs/node-api.md"));
    assert!(node_doc.contains(rule));
    assert!(node_doc.contains("forbiddenPaths"));
}

#[test]
fn test_fixture_size_cap_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::TEST_FIXTURE_SIZE_CAP;
    assert_eq!(rule, "test-fixture-size-cap");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    assert!(repo_root().join(format!("docs/rules/{rule}.md")).is_file());
    let root = repo_root();
    assert!(read(&root.join("docs/configuration/rules.md")).contains(rule));
    assert!(read(&root.join("docs/node-api.md")).contains(rule));
}

#[test]
fn github_actions_local_action_checkout_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::GITHUB_ACTIONS_LOCAL_ACTION_CHECKOUT;
    assert_eq!(rule, "github-actions-local-action-checkout");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    assert!(repo_root().join(format!("docs/rules/{rule}.md")).is_file());
}

#[test]
fn github_actions_run_command_policy_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::GITHUB_ACTIONS_RUN_COMMAND_POLICY;
    assert_eq!(rule, "github-actions-run-command-policy");
    docs_coverage_rules::assert_run_command_policy_docs(&repo_root(), rule);
}

#[test]
fn dependabot_cooldown_policy_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::DEPENDABOT_COOLDOWN_POLICY;
    assert_eq!(rule, "dependabot-cooldown-policy");
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    for path in [
        "docs/rules/README.md",
        "docs/rules/dependabot-cooldown-policy.md",
        "docs/configuration/rules.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/feature-parity.md",
        "plugins/no-mistakes/skills/no-mistakes/references/limits-and-fallbacks.md",
    ] {
        assert!(read(&root.join(path)).contains(rule), "{path}");
    }
}

#[test]
fn cross_package_relative_imports_are_documented() {
    let rule = no_mistakes::codebase::rules::NO_CROSS_PACKAGE_RELATIVE_IMPORTS;
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    assert!(root.join(format!("docs/rules/{rule}.md")).is_file());
    for path in [
        "docs/rules/README.md",
        "docs/configuration/rules.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/feature-parity.md",
        "plugins/no-mistakes/skills/no-mistakes/references/limits-and-fallbacks.md",
    ] {
        assert!(read(&root.join(path)).contains(rule), "{path}");
    }
}

#[test]
fn vitest_default_excludes_is_in_the_public_rule_inventory() {
    let rule = "vitest-default-excludes";
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    for path in [
        "docs/rules/vitest-default-excludes.md",
        "docs/configuration/rules.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/feature-parity.md",
    ] {
        let content = read(&root.join(path));
        assert!(content.contains(rule), "{path}");
        assert!(content.contains("options.files"), "{path}");
    }
}

#[test]
fn vitest_project_policy_is_in_the_public_rule_inventory() {
    let rule = "vitest-project-policy";
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    assert!(repo_root().join(format!("docs/rules/{rule}.md")).is_file());
}

#[test]
fn vitest_project_policy_documents_required_exclude_globs() {
    let root = repo_root();
    for path in [
        "docs/rules/vitest-project-policy.md",
        "docs/configuration/rules.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/feature-parity.md",
    ] {
        assert!(
            read(&root.join(path)).contains("requiredExcludeGlobs"),
            "{path}"
        );
    }
}

#[test]
fn postgres_materialized_view_shape_is_in_the_public_rule_inventory() {
    let rule = no_mistakes::codebase::rules::POSTGRES_MATERIALIZED_VIEW_SHAPE;
    assert_eq!(rule, "postgres-materialized-view-shape");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    assert!(repo_root().join(format!("docs/rules/{rule}.md")).is_file());
}

#[test]
fn bounded_statement_body_policy_is_documented_for_cli_and_node() {
    let root = repo_root();
    for path in [
        "docs/rules/postgres-bounded-statements.md",
        "docs/node-api.md",
        "docs/configuration/rules.md",
    ] {
        assert!(
            read(&root.join(path)).contains("materializedBodies"),
            "{path}"
        );
    }
}

#[test]
fn grouping_advice_documents_all_codes_and_advisory_contract() {
    let rule = no_mistakes::codebase::rules::POSTGRES_GROUPING_NOT_AGGREGATION;
    assert_eq!(rule, "postgres-grouping-not-aggregation");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    let page = read(&repo_root().join(format!("docs/rules/{rule}.md")));
    for contract in [
        "all-of-candidate-pool",
        "distinct-over-unique-key",
        "distinct-over-union",
        "distinct-join-filter",
        "aggregate-existence-test",
        "advisories",
        "target",
        "schemaCatalogPath",
        "options.fail",
        "no-mistakes-disable-next-line",
        "every shape",
        "benchmark",
    ] {
        assert!(page.contains(contract), "missing {contract}");
    }
}

#[test]
fn vite_optimize_deps_nested_imports_documents_scope_options() {
    let rule = no_mistakes::codebase::rules::VITE_OPTIMIZE_DEPS_NESTED_IMPORTS;
    assert_eq!(rule, "vite-optimize-deps-nested-imports");
    assert!(rule_ids_from_source(&repo_root()).contains(rule));
    let root = repo_root();
    for path in [
        "docs/rules/vite-optimize-deps-nested-imports.md",
        "docs/configuration/rules.md",
        "docs/node-api.md",
        "plugins/no-mistakes/skills/no-mistakes/SKILL.md",
    ] {
        let body = read(&root.join(path));
        assert!(body.contains("configFiles"), "{path}");
        assert!(body.contains("packages"), "{path}");
        assert!(body.contains("includeSources"), "{path}");
        assert!(body.contains("ignorePackages"), "{path}");
    }
}

#[test]
fn postgres_lock_inventory_has_rule_command_and_node_documentation() {
    let root = repo_root();
    let rule = "postgres-lock-inventory";
    assert!(rule_ids_from_source(&root).contains(rule));
    for path in [
        "docs/rules/postgres-lock-inventory.md",
        "docs/cli/postgres-lock-inventory.md",
    ] {
        let page = read(&root.join(path));
        assert!(page.contains("baseline"), "{path}");
        assert!(page.contains("baselineUnanalyzed"), "{path}");
        assert!(page.contains("unanalyzedByReason"), "{path}");
        assert!(page.contains("unanalyzedByDirectory"), "{path}");
        assert!(page.contains("unresolved"), "{path}");
    }
    assert!(read(&root.join("docs/node-api.md")).contains("postgresLockInventory"));
}

#[test]
fn test_file_hygiene_eslint_rules_have_public_docs_and_options() {
    let root = repo_root();
    let registry = read(&root.join("packages/eslint-plugin-no-mistakes/src/index.js"));
    let catalog = read(&root.join("docs/eslint-rules/README.md"));
    let plugin_docs = read(&root.join("docs/eslint-plugin.md"));
    for rule in [
        "test-no-fixed-temp-paths",
        "test-no-typed-enqueue-fake",
        "test-no-wall-time-assertions",
    ] {
        assert!(registry.contains(&format!("\"{rule}\": require(\"./rules/{rule}\")")));
        assert!(catalog.contains(&format!("{rule}.md")));
        assert!(plugin_docs.contains(&format!("### `{rule}`")));
        let doc = read(&root.join(format!("docs/eslint-rules/{rule}.md")));
        for section in ["Why", "Options", "Suppression", "Related"] {
            assert!(
                doc.to_ascii_lowercase()
                    .contains(&section.to_ascii_lowercase()),
                "{rule}: {section}"
            );
        }
    }
    let types = read(&root.join("packages/eslint-plugin-no-mistakes/src/index.d.ts"));
    assert!(types.contains("export interface TestNoWallTimeAssertionsOptions"));
    assert!(types.contains("allowedPaths?: string[]"));
}

#[test]
fn changes_diff_and_semantic_test_routing_have_public_docs() {
    let root = repo_root();
    for page in ["docs/cli/changes-diff.md", "docs/configuration/changes.md"] {
        let body = read(&root.join(page));
        assert!(body.contains("changes"), "{page}");
        assert!(
            body.contains("keysOf") || body.contains("schemaVersion"),
            "{page}"
        );
    }
    for page in [
        "docs/cli/tests-plan.md",
        "docs/node-api.md",
        "plugins/no-mistakes/skills/no-mistakes/references/tests.md",
    ] {
        assert!(read(&root.join(page)).contains("changes"), "{page}");
    }
    assert!(read(&root.join("packages/no-mistakes/changes-types.d.ts"))
        .contains("export interface ChangesDiffOptions"));
}
