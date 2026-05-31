use no_mistakes::codebase::{rules, unique_exports};
use no_mistakes::playwright::rules as playwright_rules;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn joined_docs(dir: &Path) -> String {
    let mut body = String::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            body.push_str(&read(&path));
            body.push('\n');
        }
    }
    body
}

#[test]
fn cli_leaf_commands_have_docs() {
    let root = repo_root();
    let cli_docs = joined_docs(&root.join("docs/cli"));
    let commands = [
        "dependencies",
        "dependents",
        "related",
        "symbols",
        "fetches",
        "check",
        "tests-plan",
        "tests-impact",
        "tests-why",
        "tests-comment",
        "tests-graph",
        "playwright-check",
        "playwright-edges",
        "playwright-related",
        "playwright-tests",
        "react-analyze",
        "react-check",
        "queues-edges",
        "queues-related",
        "queues-check",
        "server-routes",
        "server-edges",
        "server-related",
    ];
    for command in commands {
        let file = format!("{command}.md");
        let path = root.join("docs/cli").join(&file);
        assert!(path.exists(), "missing CLI doc {}", path.display());
        assert!(cli_docs.contains(&file), "docs/cli/*.md must link {file}");
    }
}

#[test]
fn no_mistakes_rules_have_docs() {
    let root = repo_root();
    let index = read(&root.join("docs/rules/README.md"));
    let rule_ids = [
        rules::AGENTS_MD_MAX_SIZE,
        rules::BANNED_RENAMED_FILES,
        rules::DOC_CONSISTENCY,
        rules::FILE_EXTENSION_POLICY,
        rules::FORBIDDEN_DEPENDENCIES,
        rules::LOCKFILE_ALLOWLIST,
        rules::NEXTJS_NO_API_ROUTES,
        rules::NEXTJS_NO_CACHING,
        rules::NO_EMPTY_OR_COMMENTS_ONLY_FILES,
        rules::NO_GIT_IDENTITY_MUTATION,
        rules::PACKAGE_JSON_REGISTRY_ONLY,
        playwright_rules::PLAYWRIGHT_COVERAGE,
        playwright_rules::PLAYWRIGHT_UNIQUE_HTML_IDS,
        playwright_rules::PLAYWRIGHT_UNIQUE_TEST_IDS,
        rules::REQUIRE_FILES_IN_SUBDIRS,
        rules::REQUIRE_STORYBOOK_STORIES,
        rules::REQUIRE_TEST_PER_SUBDIR,
        rules::REQUIRED_DOC_SECTION,
        rules::REQUIRED_LOCAL_DOCS,
        rules::RUST_MAX_LINES_PER_FILE,
        rules::RUST_NO_INLINE_ALLOWS,
        rules::RUST_NO_INLINE_TESTS,
        rules::SERVER_ROUTE_CLIENT_BOUNDARY,
        rules::SHELLCHECK_RUNNER,
        rules::STRICT_PACKAGE_LAYOUT,
        rules::TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
        rules::TSCONFIG_ALIAS_FOLDER_MAPPING,
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
