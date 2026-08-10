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
        None => (format!("{variant}.md"), false),
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

fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            paths.extend(rust_sources(&path));
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            paths.push(path);
        }
    }
    paths.sort();
    paths
}

fn subcommand_enums(source: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut search_from = 0;
    while let Some(relative) = source[search_from..].find("#[command(subcommand)]") {
        let attribute_start = search_from + relative;
        let after_attribute = attribute_start + "#[command(subcommand)]".len();
        let field_end = source[after_attribute..]
            .find('}')
            .map(|offset| after_attribute + offset)
            .unwrap_or(source.len());
        let field = &source[after_attribute..field_end];
        let Some(command_field) = field.find("command:") else {
            search_from = after_attribute;
            continue;
        };
        let enum_name = field[command_field + "command:".len()..]
            .split(|character: char| character == ',' || character == ';' || character == '}')
            .next()
            .unwrap()
            .trim()
            .to_string();
        let parent = source[..attribute_start]
            .rfind("struct ")
            .and_then(|offset| {
                source[offset + "struct ".len()..]
                    .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                    .next()
            })
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| panic!("clap subcommand attribute has no containing struct"))
            .to_string();
        result.push((parent, enum_name));
        search_from = after_attribute;
    }
    result
}

fn enum_block<'a>(source: &'a str, enum_name: &str) -> Option<&'a str> {
    let marker = format!("enum {enum_name}");
    let mut search_from = 0;
    while let Some(relative) = source[search_from..].find(&marker) {
        let start = search_from + relative;
        let after_name = start + marker.len();
        if source[after_name..]
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            search_from = after_name;
            continue;
        }
        let open = source[after_name..].find('{')? + after_name;
        let mut depth = 0;
        for (offset, character) in source[open..].char_indices() {
            match character {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&source[open + 1..open + offset]);
                    }
                }
                _ => {}
            }
        }
        return None;
    }
    None
}

fn enum_variants(block: &str) -> Vec<String> {
    block
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                return None;
            }
            let name = line
                .split(|character: char| {
                    character == '('
                        || character == '{'
                        || character == ','
                        || character.is_whitespace()
                })
                .next()?;
            if name.chars().next()?.is_ascii_uppercase()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
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
