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
    consumer_identifier_paths, enum_block, enum_variants, kebab_case, reachable_cli_pages,
    rust_sources, subcommand_enums,
};

#[path = "support/docs_coverage_commands.rs"]
mod docs_coverage_commands;
#[path = "support/docs_coverage_rules.rs"]
mod docs_coverage_rules;

#[test]
fn identifier_inventory_includes_root_and_agent_documents() {
    // Saved root/agent documents protect scan scope; ignored build output stays out.
    let source = repo_root().join("test-cases/docs/identifier-inventory/fixture");
    let fixture = saved_fixture::materialize_path(&source);
    let root = fixture.path();
    std::fs::rename(root.join(".git.fixture"), root.join(".git")).unwrap();
    assert!(root.join(".git/config").is_file());
    assert!(root.join("ignored.md").is_file());
    let paths: BTreeSet<_> = consumer_identifier_paths(root)
        .into_iter()
        .map(|path| path.strip_prefix(root).unwrap().to_owned())
        .collect();
    assert!(paths.contains(Path::new("README.md")));
    assert!(paths.contains(Path::new("skills/no-mistakes/SKILL.md")));
    assert!(paths.contains(Path::new("skills/no-mistakes/references/example.md")));
    assert!(!paths.contains(Path::new("ignored.md")));
    assert!(!paths.iter().any(|path| path.starts_with(".git")));
}

#[test]
fn consumer_identifiers_stay_outside_the_denylist() {
    const CONSUMER_TOKENS: &[&str] = &["voucha", "@data-stores/valkey", "voucha.ai"];
    const DENYLIST_FILE: &str = "crates/no-mistakes/tests/docs_coverage.rs";

    let root = repo_root();
    let denylist = read(&root.join(DENYLIST_FILE)).to_ascii_lowercase();
    let declaration =
        format!("const CONSUMER_TOKENS: &[&str] = &{:?};", CONSUMER_TOKENS).to_ascii_lowercase();
    assert!(
        denylist.contains(&declaration),
        "the deny-list exception must remain the documented token declaration"
    );
    for (token, expected_occurrences) in CONSUMER_TOKENS.iter().zip([2, 1, 1]) {
        assert_eq!(
            denylist.matches(token).count(),
            expected_occurrences,
            "the deny-list may mention each consumer token only in its declaration"
        );
    }

    let mut violations = Vec::new();
    for path in consumer_identifier_paths(&root) {
        let relative = path.strip_prefix(&root).unwrap_or(&path);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", relative.display()));
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let lower = text.to_ascii_lowercase();
        if relative == Path::new(DENYLIST_FILE) {
            continue;
        }
        for (line, contents) in lower.lines().enumerate() {
            for token in CONSUMER_TOKENS {
                if contents.contains(token) {
                    violations.push(format!(
                        "{}:{} contains a forbidden consumer identifier",
                        relative.display(),
                        line + 1
                    ));
                }
            }
        }
    }
    violations.sort();
    assert!(
        violations.is_empty(),
        "consumer identifiers may occur only in the documented deny-list:\n{}",
        violations.join("\n")
    );
}

#[test]
fn review_found_doc_regressions_stay_fixed() {
    let root = repo_root();
    let read_root = |path: &str| read(&root.join(path));

    let readme = read_root("README.md");
    assert!(readme.contains("<playwright\\|vitest>"));

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

    let limits = read_root("skills/no-mistakes/references/limits-and-fallbacks.md");
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
        "not started",
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

    let root_readme = read_root("README.md");
    assert!(
        root_readme.contains("(docs/feature-parity.md)"),
        "README.md must link docs/feature-parity.md"
    );
    assert!(root_readme.contains("(docs/why.md)"));

    let why = read_root("docs/why.md");
    assert!(why.contains("Three jobs"));
    assert!(why.contains("Mistakes this prevents"));
}
