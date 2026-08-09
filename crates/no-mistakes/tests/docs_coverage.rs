use no_mistakes::codebase::{rules, unique_exports};
use no_mistakes::playwright::rules as playwright_rules;
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

#[test]
fn cli_commands_have_docs() {
    let root = repo_root();
    let source = read(&root.join("crates/no-mistakes/src/main.rs"));
    let index = read(&root.join("docs/cli/README.md"));
    let command_block = source
        .split_once("enum Command {")
        .and_then(|(_, rest)| rest.split_once("\n}\n"))
        .map(|(block, _)| block)
        .expect("main.rs must define a closed Command enum");
    assert!(
        !command_block.lines().any(|line| line.contains("name =")),
        "a clap command name override needs an explicit docs-coverage mapping"
    );

    let commands = command_block
        .lines()
        .filter_map(|line| {
            let name = line.trim().split_once('(')?.0;
            if name.is_empty() || name.starts_with('#') || name.starts_with("///") {
                return None;
            }
            Some(kebab_case(name))
        })
        .collect::<Vec<_>>();
    assert!(
        !commands.is_empty(),
        "Command enum inventory must not be empty"
    );

    for command in commands {
        let file = format!("{command}.md");
        let path = root.join("docs/cli").join(&file);
        assert!(path.exists(), "missing CLI doc {}", path.display());
        assert!(
            index.contains(&format!("({file})")),
            "docs/cli/README.md must index {file}"
        );
    }

    // Every leaf page must be reachable from the CLI index or its command
    // group page. Follow only links rooted under docs/cli so an orphan page
    // cannot make itself appear reachable by containing its own filename.
    let cli_dir = root.join("docs/cli");
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

fn reachable_cli_pages(cli_dir: &Path) -> BTreeSet<String> {
    let cli_dir = cli_dir.canonicalize().unwrap();
    let mut seen = BTreeSet::new();
    let mut pending = VecDeque::from([cli_dir.join("README.md")]);
    while let Some(path) = pending.pop_front() {
        let Ok(relative) = path.strip_prefix(&cli_dir) else {
            continue;
        };
        let relative = relative.to_string_lossy().into_owned();
        if !seen.insert(relative) {
            continue;
        }
        let body = read(&path);
        let mut remaining = body.as_str();
        while let Some(start) = remaining.find("](") {
            remaining = &remaining[start + 2..];
            let Some(end) = remaining.find(')') else {
                break;
            };
            let target = remaining[..end].split('#').next().unwrap_or_default();
            remaining = &remaining[end + 1..];
            if target.is_empty() || target.starts_with("http") {
                continue;
            }
            let target_path = path.parent().unwrap().join(target);
            if target_path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let Ok(target_path) = target_path.canonicalize() else {
                continue;
            };
            if target_path.starts_with(&cli_dir) {
                pending.push_back(target_path);
            }
        }
    }
    seen
}

fn kebab_case(value: &str) -> String {
    let mut result = String::new();
    for (index, character) in value.chars().enumerate() {
        if character.is_uppercase() && index != 0 {
            result.push('-');
        }
        result.extend(character.to_lowercase());
    }
    result
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
    for export in exports {
        assert!(
            docs.lines()
                .any(|line| line.starts_with('|') && line.contains(&format!("`{export}`"))),
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
        rules::DOC_CONSISTENCY,
        rules::FILE_EXTENSION_POLICY,
        rules::FORBIDDEN_DEPENDENCIES,
        rules::FORBIDDEN_WORKSPACE_CLOSURE,
        rules::INTEGRATION_TEST_NO_MOCKS,
        rules::LOCKFILE_ALLOWLIST,
        rules::MARKDOWN_LINK_DISPLAY_TEXT,
        rules::MARKDOWN_MERMAID_VALIDATION,
        rules::MARKDOWN_REACHABILITY,
        rules::MARKDOWN_STRUCTURE_BUDGET,
        rules::NEXTJS_NO_API_ROUTES,
        rules::NEXTJS_NO_CACHING,
        rules::NO_EMPTY_OR_COMMENTS_ONLY_FILES,
        rules::NO_GIT_IDENTITY_MUTATION,
        rules::PACKAGE_JSON_REGISTRY_ONLY,
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
        rules::TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
        rules::TSCONFIG_ALIAS_FOLDER_MAPPING,
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
}
