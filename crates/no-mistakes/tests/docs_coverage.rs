use no_mistakes::codebase::{rules, unique_exports};
use no_mistakes::playwright::rules as playwright_rules;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[path = "support/docs_coverage_cli_helpers.rs"]
mod cli_docs_helpers;
use cli_docs_helpers::{
    enum_block, enum_variants, kebab_case, reachable_cli_pages, rust_sources, subcommand_enums,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

#[test]
fn cli_commands_have_docs() {
    let root = repo_root();
    let cli_dir = root.join("docs/cli");
    let index = read(&cli_dir.join("README.md"));
    let source_dir = root.join("crates/no-mistakes/src");

    // Inventory every clap subcommand field from source. This includes the
    // top-level `Command` enum and nested enums such as `TestsCommand`; a
    // guard that only knows about main.rs can silently lose `tests plan` and
    // other leaf pages when a command is added or its page is deleted.
    let mut inventories = Vec::new();
    for source_path in rust_sources(&source_dir) {
        let source = read(&source_path);
        for (parent, enum_name) in subcommand_enums(&source) {
            let block = enum_block(&source, &enum_name).unwrap_or_else(|| {
                panic!(
                    "{}: clap subcommand type `{enum_name}` must have an enum body",
                    source_path.display()
                )
            });
            assert!(
                !block.lines().any(|line| line.contains("name =")),
                "{}: a clap command name override needs an explicit docs-coverage mapping",
                source_path.display()
            );
            let variants = enum_variants(block);
            assert!(
                !variants.is_empty(),
                "{}: {enum_name} command inventory must not be empty",
                source_path.display()
            );
            inventories.push((parent, variants));
        }
    }
    assert!(
        !inventories.is_empty(),
        "source inventory must find at least one clap subcommand enum"
    );

    for (parent, variants) in inventories {
        let Some(prefix) = parent.strip_suffix("Args").map(kebab_case) else {
            // `Cli` is the one top-level parser struct; its command pages are
            // rooted directly at docs/cli and have no group prefix.
            assert_eq!(parent, "Cli", "unexpected clap parser struct `{parent}`");
            for variant in variants {
                assert_cli_page(&cli_dir, &index, &variant, None, 1);
            }
            continue;
        };

        let group_file = format!("{prefix}.md");
        let group_path = cli_dir.join(&group_file);
        assert!(
            group_path.exists(),
            "missing CLI group doc {}",
            group_path.display()
        );
        assert!(
            index.contains(&format!("({group_file})")),
            "docs/cli/README.md must index {group_file}"
        );

        let variant_count = variants.len();
        for variant in variants {
            assert_cli_page(
                &cli_dir,
                &read(&group_path),
                &variant,
                Some(&prefix),
                variant_count,
            );
        }
    }

    // Every leaf page must be reachable from the CLI index or its command
    // group page. Follow only links rooted under docs/cli so an orphan page
    // cannot make itself appear reachable by containing its own filename.
    let linked_pages = reachable_cli_pages(&cli_dir);
    for entry in std::fs::read_dir(root.join("docs/cli")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md")
            || path.file_name().and_then(|name| name.to_str()) == Some("README.md")
        {
            continue;
        }
        let file = path.file_name().unwrap().to_string_lossy();
        assert!(
            linked_pages.contains(file.as_ref()),
            "CLI page {file} is not linked by a CLI index or command group"
        );
    }
}

fn assert_cli_page(
    cli_dir: &Path,
    parent_body: &str,
    variant: &str,
    prefix: Option<&str>,
    variant_count: usize,
) {
    let variant = kebab_case(variant);
    let (file, indexed_by_parent) = match prefix {
        Some(prefix) => {
            let leaf_file = format!("{prefix}-{variant}.md");
            let leaf_path = cli_dir.join(&leaf_file);
            if leaf_path.exists() {
                (leaf_file, true)
            } else {
                // A one-command group may document its only leaf directly on
                // the group page (currently `lockfile diff`). If a second
                // variant is added, the caller's per-variant lookup will no
                // longer permit this fallback without a dedicated leaf page.
                let group_file = format!("{prefix}.md");
                assert_cli_group_has_one_leaf(parent_body, prefix, &group_file, variant_count);
                (group_file, false)
            }
        }
        None => (format!("{variant}.md"), true),
    };
    let path = cli_dir.join(&file);
    assert!(path.exists(), "missing CLI doc {}", path.display());
    if indexed_by_parent {
        assert!(
            parent_body.contains(&format!("({file})")),
            "CLI group doc must index {file}"
        );
    }
}

fn assert_cli_group_has_one_leaf(
    parent_body: &str,
    prefix: &str,
    group_file: &str,
    variant_count: usize,
) {
    assert_eq!(
        variant_count, 1,
        "{group_file} has multiple subcommands; every leaf needs a dedicated CLI page"
    );
    let linked_children = parent_body
        .lines()
        .filter_map(|line| {
            let target = line.split_once("](")?.1.split_once(')')?.0;
            Some(target.to_string())
        })
        .filter(|target| target.ends_with(".md") && target.starts_with(&format!("{prefix}-")))
        .count();
    assert_eq!(
        linked_children, 0,
        "{group_file} has linked leaf pages; missing {prefix}-<command>.md must be fixed explicitly"
    );
}

#[test]
fn node_runtime_exports_have_api_docs() {
    let root = repo_root();
    let source = read(&root.join("packages/no-mistakes/index.js"));
    let docs = read(&root.join("docs/node-api.md"));
    let exports = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("module.exports."))
        .filter_map(|assignment| assignment.split_once(' ').map(|(name, _)| name))
        .collect::<Vec<_>>();
    assert!(
        !exports.is_empty(),
        "runtime export inventory must not be empty"
    );
    let runtime_inventory = docs
        .split_once("| Runtime export | API |\n")
        .and_then(|(_, rest)| rest.split_once("\n\n").map(|(table, _)| table))
        .expect("docs/node-api.md must contain a complete runtime export inventory table");
    let source_exports = exports.iter().copied().collect::<BTreeSet<_>>();
    let documented_rows = runtime_inventory
        .lines()
        .filter_map(|line| {
            line.strip_prefix("| `")?
                .split_once("` |")
                .map(|(name, api)| (name, api.trim().trim_end_matches('|').trim()))
        })
        .collect::<Vec<_>>();
    for (export, api) in &documented_rows {
        assert!(
            !api.is_empty(),
            "runtime export `{export}` needs an API mapping"
        );
    }
    let documented_exports = documented_rows
        .iter()
        .map(|(name, _)| *name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        documented_exports, source_exports,
        "runtime export inventory must exactly match packages/no-mistakes/index.js"
    );
    for export in source_exports {
        assert!(
            runtime_inventory.contains(&format!("| `{export}` |")),
            "docs/node-api.md must map runtime export `{export}`"
        );
    }
}

#[test]
fn no_mistakes_rules_have_docs() {
    let root = repo_root();
    let index = read(&root.join("docs/rules/README.md"));
    let rule_ids = [
        rules::AGENTS_MD_MAX_SIZE,
        rules::BANNED_RENAMED_FILES,
        rules::CSHARP_MAX_LINES_PER_FILE,
        rules::DOC_CONSISTENCY,
        rules::FILE_EXTENSION_POLICY,
        rules::GITHUB_ACTIONS_ACTION_TIMEOUT_PAIR,
        rules::GITHUB_ACTIONS_COMPOSITE_STEP_SCHEMA,
        rules::GITHUB_ACTIONS_JOB_TIMEOUTS,
        rules::GITHUB_ACTIONS_TEST_TIMEOUT_LITERALS,
        rules::VERSION_PIN_CONSISTENCY,
        rules::FORBIDDEN_DEPENDENCIES,
        rules::FORBIDDEN_WORKSPACE_CLOSURE,
        rules::INTEGRATION_TEST_NO_MOCKS,
        rules::LOCKFILE_ALLOWLIST,
        rules::MARKDOWN_CHILD_LINKS,
        rules::MARKDOWN_EVAL_TESTS,
        rules::MARKDOWN_LINK_DISPLAY_TEXT,
        rules::MARKDOWN_MERMAID_VALIDATION,
        rules::MARKDOWN_REACHABILITY,
        rules::MARKDOWN_STRUCTURE_BUDGET,
        rules::NEXTJS_NO_API_ROUTES,
        rules::NEXTJS_NO_CACHING,
        rules::NEXTJS_REDIRECT_DESTINATIONS,
        rules::NO_EMPTY_OR_COMMENTS_ONLY_FILES,
        rules::NO_GIT_IDENTITY_MUTATION,
        rules::NO_RAW_EPHEMERAL_PORT,
        rules::PACKAGE_JSON_REGISTRY_ONLY,
        rules::POSTGRES_CONSTRAINT_VALIDATE,
        rules::POSTGRES_NO_ADD_COLUMN,
        rules::POSTGRES_REQUIRE_NAMED_CONSTRAINTS,
        rules::POSTGRES_REQUIRE_FK_ON_DELETE,
        rules::POSTGRES_FK_INDEX,
        rules::POSTGRES_LOCK_ORDERING,
        rules::POSTGRES_NO_OFFSET,
        rules::POSTGRES_REQUIRE_QUERY_ANNOTATION,
        rules::POSTGRES_NO_GENERATED_COLUMN_WRITES,
        rules::POSTGRES_REDUNDANT_INDEX,
        playwright_rules::PLAYWRIGHT_COVERAGE,
        playwright_rules::PLAYWRIGHT_PREFER_TEST_ID_LOCATORS,
        playwright_rules::PLAYWRIGHT_UNIQUE_HTML_IDS,
        playwright_rules::PLAYWRIGHT_UNIQUE_TEST_IDS,
        rules::PRODUCTION_DEPENDENCY_DECLARATIONS,
        rules::REQUIRE_FILES_IN_SUBDIRS,
        rules::REQUIRE_STORYBOOK_STORIES,
        rules::REQUIRE_TEST_PER_SUBDIR,
        rules::REQUIRED_ENTRYPOINT_REACHABILITY,
        rules::REQUIRED_DOC_SECTION,
        rules::REQUIRED_LOCAL_DOCS,
        rules::RUST_MAX_LINES_PER_FILE,
        rules::RUST_NO_INLINE_ALLOWS,
        rules::RUST_NO_INLINE_TESTS,
        rules::SERVER_ROUTE_CLIENT_BOUNDARY,
        rules::SHELLCHECK_RUNNER,
        rules::STRICT_PACKAGE_LAYOUT,
        rules::TEST_EMAIL_DOMAIN_POLICY,
        rules::TEST_NO_DEPENDENCY_PINS,
        rules::TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
        rules::TSCONFIG_ALIAS_FOLDER_MAPPING,
        rules::TSCONFIG_FILE_COVERAGE,
        rules::TSCONFIG_GATE_COVERAGE,
        unique_exports::RULE_ID,
        rules::VITEST_TEST_CORRESPONDENCE,
    ];
    for rule_id in rule_ids {
        let file = format!("{rule_id}.md");
        let path = root.join("docs/rules").join(&file);
        assert!(path.exists(), "missing rule doc {}", path.display());
        assert!(
            index.contains(&file),
            "docs/rules/README.md must link {file}"
        );
        let body = read(&path);
        assert!(
            body.contains("Counterexample:"),
            "{file} needs a counterexample"
        );
        assert!(body.contains("Fix:"), "{file} needs fix guidance");
    }
}

#[test]
fn graph_edge_kinds_are_documented() {
    let root = repo_root();
    let body = read(&root.join("docs/graph-edges.md"));
    let edge_kinds = [
        "import",
        "type-import",
        "dynamic-import",
        "require",
        "test",
        "route",
        "queue-enqueue",
        "queue-worker",
        "route-test",
        "layout",
        "md",
        "workspace",
        "package",
        "ci",
        "http",
        "process",
        "asset",
        "react-render",
        "selector",
        "dotnet-using",
        "dotnet-ref",
        "dotnet-project",
        "python-import",
        "go-import",
        "rust-use",
        "ruby-require",
        "php-use",
        "trpc-call",
        "trpc-procedure",
    ];
    for edge_kind in edge_kinds {
        assert!(
            body.contains(&format!("`{edge_kind}`")),
            "missing {edge_kind}"
        );
    }
    assert!(body.contains("Examples And Counterexamples"));
    assert!(body.contains("Intentional Limits"));
}

#[test]
fn rule_docs_use_supported_option_examples() {
    let root = repo_root();
    let cases = [
        (
            "require-files-in-subdirs.md",
            ["packages:", "requiredFiles:", "requireAnyOf:"].as_slice(),
            ["roots:", "files:"].as_slice(),
        ),
        (
            "strict-package-layout.md",
            [
                "packages:",
                "sourceExtension:",
                "allowedRootFiles:",
                "allowedSubdirs:",
            ]
            .as_slice(),
            ["roots:", "requiredFiles:"].as_slice(),
        ),
        (
            "banned-renamed-files.md",
            ["bannedBasenames:", "name:", "message:", "extensions:"].as_slice(),
            ["banned:", "from:", "to:"].as_slice(),
        ),
        (
            "file-extension-policy.md",
            ["allowlist:", "scopes:", "bannedExtensions:"].as_slice(),
            ["allowed:"].as_slice(),
        ),
        (
            "require-storybook-stories.md",
            ["stories:", "includeAllReactNamedExports:"].as_slice(),
            [].as_slice(),
        ),
        (
            "tsconfig-alias-folder-mapping.md",
            ["tsconfig:", "mappings:", "prefix:", "root:"].as_slice(),
            [].as_slice(),
        ),
        (
            "unique-exports.md",
            ["uniqueAcrossTypesAndValues:"].as_slice(),
            ["strict:"].as_slice(),
        ),
        (
            "package-json-registry-only.md",
            ["scopes:", "lockfile:"].as_slice(),
            ["registry:"].as_slice(),
        ),
    ];

    for (file, required, forbidden) in cases {
        let body = read(&root.join("docs/rules").join(file));
        for needle in required {
            assert!(body.contains(needle), "{file} missing `{needle}`");
        }
        for needle in forbidden {
            assert!(!body.contains(needle), "{file} still contains `{needle}`");
        }
    }
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
    assert!(eslint_plugin.contains(r#""named" \| "default""#));

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

    let root_readme = read_root("README.md");
    assert!(
        root_readme.contains("(docs/feature-parity.md)"),
        "README.md must link docs/feature-parity.md"
    );
}
