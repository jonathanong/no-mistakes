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
fn tagged_git_paths_separate_index_entries_from_untracked_entries() {
    let views = parse_git_tagged_paths(
        b"broken\0H \0H tracked file.mts\0S sparse.mts\0? untracked file.mts\0K killed.mts\0R deleted.mts\0",
    );

    assert_eq!(
        views
            .visible
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>(),
        vec![
            PathBuf::from("killed.mts"),
            PathBuf::from("sparse.mts"),
            PathBuf::from("tracked file.mts"),
            PathBuf::from("untracked file.mts"),
        ]
    );
    assert!(views.visible.iter().all(|entry| entry.index_kind.is_none()));
    assert_eq!(
        views.tracked,
        vec![
            PathBuf::from("sparse.mts"),
            PathBuf::from("tracked file.mts"),
        ]
    );
}

#[cfg(unix)]
#[test]
fn tagged_git_paths_preserve_non_utf8_path_bytes() {
    use std::os::unix::ffi::OsStrExt;

    let views = parse_git_tagged_paths(b"H src/non-utf8-\xff.mts\0");

    assert_eq!(views.visible.len(), 1);
    assert_eq!(
        views.visible[0].path.as_os_str().as_bytes(),
        b"src/non-utf8-\xff.mts"
    );
    assert!(views.visible[0].index_kind.is_none());
    assert_eq!(views.tracked, vec![views.visible[0].path.clone()]);
}

#[test]
fn staged_git_paths_classify_regular_files_and_symlinks_from_index_mode() {
    use super::super::file_inventory::GitIndexKind;

    let views = parse_git_tagged_paths(
        b"H 100644 abcdef 0\ttracked file.mts\0? untracked.mts\0K killed.mts\0H 120000 abcdef 0\tlink.mts\0H 100755 abcdef 0\texec.mts\0H 160000 abcdef 0\tsubmodule\0",
    );

    assert_eq!(
        views
            .visible
            .iter()
            .map(|entry| (entry.path.clone(), entry.index_kind))
            .collect::<Vec<_>>(),
        vec![
            (PathBuf::from("exec.mts"), Some(GitIndexKind::RegularFile)),
            (PathBuf::from("killed.mts"), None),
            (PathBuf::from("link.mts"), Some(GitIndexKind::Symlink)),
            (PathBuf::from("submodule"), None),
            (
                PathBuf::from("tracked file.mts"),
                Some(GitIndexKind::RegularFile)
            ),
            (PathBuf::from("untracked.mts"), None),
        ]
    );
    assert_eq!(
        views.tracked,
        vec![
            PathBuf::from("exec.mts"),
            PathBuf::from("link.mts"),
            PathBuf::from("submodule"),
            PathBuf::from("tracked file.mts"),
        ]
    );
}

#[test]
fn staged_git_paths_drop_deleted_worktree_entries() {
    use super::super::file_inventory::GitIndexKind;

    let views = parse_git_tagged_paths(
        b"H 100644 abcdef 0\tdeleted.mts\0R 100644 abcdef 0\tdeleted.mts\0H 100644 abcdef 0\tkept.mts\0",
    );

    assert_eq!(
        views
            .visible
            .iter()
            .map(|entry| (entry.path.clone(), entry.index_kind))
            .collect::<Vec<_>>(),
        vec![(PathBuf::from("kept.mts"), Some(GitIndexKind::RegularFile))]
    );
    assert_eq!(views.tracked, vec![PathBuf::from("kept.mts")]);
}

#[test]
fn staged_git_paths_do_not_trust_skip_worktree_index_mode() {
    use super::super::file_inventory::GitIndexKind;

    let views =
        parse_git_tagged_paths(b"H 100644 abcdef 0\tkept.mts\0S 100644 abcdef 0\tsparse.mts\0");

    assert_eq!(
        views
            .visible
            .iter()
            .map(|entry| (entry.path.clone(), entry.index_kind))
            .collect::<Vec<_>>(),
        vec![
            (PathBuf::from("kept.mts"), Some(GitIndexKind::RegularFile)),
            (PathBuf::from("sparse.mts"), None),
        ]
    );
    assert_eq!(
        views.tracked,
        vec![PathBuf::from("kept.mts"), PathBuf::from("sparse.mts")]
    );
}

#[test]
fn staged_git_paths_keep_literal_untracked_stage_shaped_names() {
    // `?` records are a literal path. A name that matches `--stage` metadata
    // must not be decoded into mode/object/stage.
    let views = parse_git_tagged_paths(b"? 100644 abcdef 0\tactual.mts\0");

    assert_eq!(
        views
            .visible
            .iter()
            .map(|entry| (entry.path.clone(), entry.index_kind))
            .collect::<Vec<_>>(),
        vec![(PathBuf::from("100644 abcdef 0\tactual.mts"), None)]
    );
    assert!(views.tracked.is_empty());
}

#[test]
fn staged_git_paths_do_not_trust_unmerged_index_mode() {
    let views = parse_git_tagged_paths(
        b"M 100644 abcdef 1\tconflict.mts\0M 120000 abcdef 3\tconflict.mts\0C 100644 abcdef 0\tstaged.mts\0",
    );

    assert_eq!(
        views
            .visible
            .iter()
            .map(|entry| (entry.path.clone(), entry.index_kind))
            .collect::<Vec<_>>(),
        vec![
            (PathBuf::from("conflict.mts"), None),
            (
                PathBuf::from("staged.mts"),
                Some(super::super::file_inventory::GitIndexKind::RegularFile)
            ),
        ]
    );
    assert_eq!(
        views.tracked,
        vec![PathBuf::from("conflict.mts"), PathBuf::from("staged.mts")]
    );
}

#[test]
fn staged_git_paths_cover_malformed_stage_payloads_and_dedup() {
    use super::super::file_inventory::GitIndexKind;

    // Each malformed `--stage` payload must fall back to a literal path instead
    // of panicking or inventing a replacement. Duplicate paths keep the first
    // kind unless it is missing.
    let views = parse_git_tagged_paths(
        b"H 10064 abcdef 0\tshort-mode.mts\0H 10064x abcdef 0\tnon-digit-mode.mts\0H 100644  0\tempty-object.mts\0H 100644 zzzzzz 0\tnon-hex.mts\0H 100644 abcdef 0 no-tab.mts\0H 100644 abcdef \tempty-stage.mts\0H 100644 abcdef x\tnon-digit-stage.mts\0H 100644 abcdef 0\t\0S 100644 abcdef 0\tshared.mts\0H 100644 abcdef 0\tshared.mts\0H 100644 abcdef 0\tdup.mts\0H 120000 abcdef 0\tdup.mts\0H 100644 abcdef 0\tkept.mts\0",
    );

    let visible: Vec<_> = views
        .visible
        .iter()
        .map(|entry| (entry.path.clone(), entry.index_kind))
        .collect();
    assert!(!visible.iter().any(|(path, _)| path.as_os_str().is_empty()));
    assert!(visible.contains(&(PathBuf::from("kept.mts"), Some(GitIndexKind::RegularFile))));
    assert!(visible.contains(&(PathBuf::from("shared.mts"), Some(GitIndexKind::RegularFile))));
    assert_eq!(
        visible
            .iter()
            .filter(|(path, _)| path == Path::new("dup.mts"))
            .count(),
        1
    );
    assert!(visible.contains(&(PathBuf::from("dup.mts"), Some(GitIndexKind::RegularFile))));
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
