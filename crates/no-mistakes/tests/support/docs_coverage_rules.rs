use super::{read, repo_root, rule_ids_from_source};

#[test]
fn no_mistakes_rules_have_docs() {
    let root = repo_root();
    let rule_ids = rule_ids_from_source(&root);
    rule_inventory::assert_inventory(&root, &rule_ids);
    for rule_id in rule_ids {
        let file = format!("{rule_id}.md");
        let path = root.join("docs/rules").join(&file);
        assert!(path.exists(), "missing rule doc {}", path.display());
        let body = read(&path);
        assert!(
            body.contains(&format!("# `{rule_id}`")),
            "{file} needs a rule title"
        );
        assert!(
            body.contains("rules:"),
            "{file} needs configuration guidance"
        );
        let lowercase_body = body.to_ascii_lowercase();
        for (section, markers) in [
            ("why and when", &["## why and when"][..]),
            (
                "behavior",
                &["## what it catches", "## what it requires"][..],
            ),
            (
                "valid example",
                &["## valid example", "compliant example:"][..],
            ),
            ("counterexample", &["counterexample"][..]),
            ("fix", &["fix"][..]),
            ("suppression", &["suppression"][..]),
            ("related rules", &["related rules"][..]),
        ] {
            assert!(
                markers.iter().any(|marker| lowercase_body.contains(marker)),
                "{file} needs {section} guidance"
            );
        }
        assert!(
            lowercase_body.contains("options"),
            "{file} must explain its rule-local options or state that none exist"
        );
        assert!(
            lowercase_body.contains("default"),
            "{file} must state every option default or that no default exists"
        );
    }
}

#[test]
fn graph_edge_kinds_are_documented() {
    let root = repo_root();
    let body = read(&root.join("docs/graph-edges.md"));
    let edge_kinds = [
        "assertion-owner",
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
        (
            "pnpm-overrides-ban.md",
            ["packageExtensions"].as_slice(),
            [].as_slice(),
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
