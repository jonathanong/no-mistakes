use super::*;

// ── starts_with_use_client ────────────────────────────────────────────────

#[test]
fn detects_single_quote_use_client() {
    assert!(starts_with_use_client(
        "'use client'\nexport function Foo() {}"
    ));
}

#[test]
fn detects_double_quote_use_client() {
    assert!(starts_with_use_client(
        "\"use client\"\nexport function Foo() {}"
    ));
}

#[test]
fn no_match_for_server_component() {
    assert!(!starts_with_use_client("export default function Page() {}"));
}

#[test]
fn only_checks_first_200_bytes() {
    let long_prefix = "a".repeat(210);
    let source = format!("{long_prefix}'use client'");
    assert!(!starts_with_use_client(&source));
}

// ── is_test_file ──────────────────────────────────────────────────────────

#[test]
fn detects_test_suffix_ts() {
    assert!(is_test_file("web/app/foo.test.ts"));
}

#[test]
fn detects_spec_suffix_tsx() {
    assert!(is_test_file("web/app/foo.spec.tsx"));
}

#[test]
fn detects_test_mts() {
    assert!(is_test_file("backend/foo.test.mts"));
}

#[test]
fn detects_tests_directory() {
    assert!(is_test_file("web/app/__tests__/foo.ts"));
}

#[test]
fn non_test_file_not_flagged() {
    assert!(!is_test_file("web/app/page.tsx"));
    assert!(!is_test_file("web/lib/api/server/users.ts"));
}

#[test]
fn source_helpers_cover_paths_lines_wrappers_and_property_names() {
    assert!(is_skipped_dir("node_modules"));
    assert!(!is_skipped_dir("src"));
    assert_eq!(
        relative_slash_path(Path::new("/repo"), Path::new("/repo/src\\file.ts")),
        "src/file.ts"
    );
    assert_eq!(line_number("a\nb\nc", 2), 2);

    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "const x = { plain: (value as string)!, \"quoted\": (<string>value) satisfies string, [dyn]: value };",
        SourceType::ts(),
    )
    .parse();
    let Statement::VariableDeclaration(var_decl) = &parsed.program.body[0] else {
        panic!("expected variable declaration");
    };
    let Expression::ObjectExpression(obj) = var_decl.declarations[0].init.as_ref().expect("init")
    else {
        panic!("expected object");
    };
    let mut names = Vec::new();
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        names.push(static_property_key_name(&prop.key));
        let _ = unwrap_ts_wrappers(&prop.value);
    }

    assert_eq!(names, vec![Some("plain"), Some("quoted"), None]);
    assert_eq!(normalize_discovery_path(Path::new("")), Path::new("."));
}

// ── git-aware discovery ─────────────────────────────────────────────────

#[test]
fn git_visible_files_include_tracked_and_untracked_non_ignored_files() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), ".gitignore", "dist/\n");
    write(dir.path(), "src/tracked.mts", "");
    write(dir.path(), "dist/ignored.mts", "");
    git_add_all(dir.path());
    write(dir.path(), "src/untracked.mts", "");

    let files = git_visible_files(dir.path()).unwrap();

    assert_eq!(
        files,
        vec![".gitignore", "src/tracked.mts", "src/untracked.mts"]
    );
}

#[test]
fn discover_files_falls_back_outside_git_repositories() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "src/main.mts", "");

    let files = discover_files(dir.path(), &[]);

    assert_eq!(files, vec![dir.path().join("src/main.mts")]);
}

#[test]
fn fallback_walk_paths_keep_the_request_root_spelling() {
    let request_root = Path::new("/var/folders/project");
    let walker_root = Path::new("/private/var/folders/project");
    let walker_path = walker_root.join("ignored-explicit/Button.tsx");

    assert_eq!(
        crate::codebase::ts_source::rebase_walk_path(request_root, walker_root, &walker_path),
        request_root.join("ignored-explicit/Button.tsx")
    );
}

#[test]
fn fallback_walk_includes_github_workflows() {
    let dir = fixture("ast-snippets/ts-source/hidden-walk");

    let files = walk_files(&dir, &[]);

    assert!(files
        .iter()
        .any(|path| path.ends_with(".github/workflows/ci.yml")));
    assert!(!files
        .iter()
        .any(|path| path.ends_with(".github/workflows/ignored.yml")));
    assert!(files.iter().any(|path| path.ends_with("src/main.mts")));
    assert!(!files.iter().any(|path| path.ends_with(".env")));
    assert!(files
        .iter()
        .any(|path| path.ends_with(".config/secret.mts")));
    assert!(!files
        .iter()
        .any(|path| path.ends_with(".cache/ignored.mts")));

    let files = walk_files(&dir, &[".github".to_string()]);
    assert!(!files
        .iter()
        .any(|path| path.ends_with(".github/workflows/ci.yml")));
}

#[test]
fn fallback_walk_does_not_prune_skipped_named_root() {
    let dir = fixture("ast-snippets/ts-source/dist");

    let files = walk_files(&dir, &[]);

    assert_eq!(files.len(), 1);
    assert!(files
        .iter()
        .any(|path| path.ends_with("dist/root-main.mts")));
}

#[test]
fn discover_source_files_filters_non_ts_js_extensions() {
    let dir = fixture("ast-snippets/ts-source");

    let files = discover_source_files(&dir, &[]);

    assert!(files.iter().any(|path| path.ends_with("jsx-walk-all.tsx")));
    assert!(!files.iter().any(|path| path.ends_with("plain.txt")));
}

#[test]
fn discover_files_normalizes_dot_components() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "src/main.mts", "");
    git_add_all(dir.path());

    let files = discover_files(&dir.path().join("."), &[]);

    assert_eq!(files, vec![dir.path().join("src/main.mts")]);
}

#[test]
fn discover_files_prunes_git_visible_skip_dirs() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "src/main.mts", "");
    write(dir.path(), "node_modules/pkg/index.mts", "");
    write(dir.path(), "dist/bundle.mts", "");
    git_add_all(dir.path());

    let files = discover_files(dir.path(), &[]);

    assert_eq!(files, vec![dir.path().join("src/main.mts")]);
}
