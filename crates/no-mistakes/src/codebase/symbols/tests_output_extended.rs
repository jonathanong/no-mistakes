#[test]
fn human_format_imports_show_type_tag_and_alias() {
    let mut args = fixture_args(vec!["src/type-alias-import.mts"], Format::Human);
    args.include = Include::Both;
    let out = run_capture(args);
    assert!(out.contains("(type)"), "expected (type) tag, got: {out}");
    assert!(
        out.contains("T as Aliased"),
        "expected 'T as Aliased' alias rendering, got: {out}"
    );
}

#[test]
fn human_format_multiple_files_lists_each_with_blank_separators() {
    let out = run_capture(fixture_args(vec!["src/a.mts", "src/b.mts"], Format::Human));
    assert!(out.contains("2 files"));
    assert!(out.contains("src/a.mts"));
    assert!(out.contains("src/b.mts"));
    assert!(out.contains("alpha"));
    assert!(out.contains("beta"));
}

#[test]
fn md_format_multiple_files_emits_per_file_subheadings() {
    let out = run_capture(fixture_args(vec!["src/a.mts", "src/b.mts"], Format::Md));
    assert!(out.contains("# 2 files"));
    assert!(out.contains("## `src/a.mts`"));
    assert!(out.contains("## `src/b.mts`"));
}

#[test]
fn md_format_imports_render_aliased_and_type_only() {
    let mut args = fixture_args(vec!["src/type-alias-import.mts"], Format::Md);
    args.include = Include::Both;
    let out = run_capture(args);
    assert!(out.contains("### Imports"));
    assert!(out.contains("`T` as `Aliased` from `src/types.mts`"));
    assert!(out.contains("(type-only)"));
}

#[test]
fn export_kind_default_serializes_as_default() {
    let out = run_capture(fixture_args(vec!["src/default.mts"], Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["files"][0]["exports"][0]["kind"], "default");
}

#[test]
fn md_format_reexport_uses_resolved_path() {
    let out = run_capture(fixture_args(vec!["src/index.mts"], Format::Md));
    assert!(out.contains("from `src/inner.mts`"));
    assert!(!out.contains("from `./inner.mts`"));
}

#[test]
fn md_format_import_uses_resolved_path() {
    let mut args = fixture_args(vec!["src/import-util.mts"], Format::Md);
    args.include = Include::Both;
    let out = run_capture(args);
    assert!(out.contains("from `src/util.mts`"));
    assert!(!out.contains("from `./util.mts`"));
}

#[test]
fn md_format_multiple_files_lists_root_paths_under_heading() {
    let out = run_capture(fixture_args(vec!["src/a.mts", "src/b.mts"], Format::Md));
    assert!(out.contains("# 2 files"));
    assert!(out.contains("- `src/a.mts`"));
    assert!(out.contains("- `src/b.mts`"));
}

#[test]
fn human_format_import_uses_resolved_path() {
    let mut args = fixture_args(vec!["src/import-util.mts"], Format::Human);
    args.include = Include::Both;
    let out = run_capture(args);
    assert!(out.contains("from src/util.mts"));
    assert!(!out.contains("from ./util.mts"));
}

#[test]
fn export_kind_enum_serializes_as_enum() {
    let out = run_capture(fixture_args(vec!["src/enum.mts"], Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["files"][0]["exports"][0]["kind"], "enum");
    assert_eq!(v["files"][0]["exports"][0]["name"], "Color");
}

#[test]
fn run_covers_formats_timings_and_resolution_fallbacks() {
    let root = fixture_root();

    for format in [
        Format::Json,
        Format::Md,
        Format::Yml,
        Format::Paths,
        Format::Human,
    ] {
        let mut args = args_for(&root, vec!["src/human.mts"], format);
        args.timings = true;
        run(args).unwrap();
    }

    let mut fallback = args_for(&root, vec!["does-not-exist.mts"], Format::Json);
    fallback.files = vec![PathBuf::from("src/human.mts")];
    fallback.root = Some(root.join("missing-root"));
    assert!(run(fallback).is_err());
}

#[test]
fn helper_branches_cover_root_tsconfig_and_kind_strings() {
    let cwd = fixture_root();
    assert_eq!(resolve_root(None, &cwd), cwd);
    assert_eq!(
        resolve_root(Some(Path::new("relative")), &cwd),
        cwd.join("relative")
    );
    assert_eq!(resolve_root(Some(&cwd), Path::new("/tmp")), cwd);

    let missing_tsconfig = resolve_tsconfig(None, &cwd.join("no-tsconfig-here")).unwrap();
    assert!(missing_tsconfig.paths.is_empty());
    let explicit_err = resolve_tsconfig(Some(&cwd.join("missing-tsconfig.json")), &cwd);
    assert!(explicit_err.is_err());

    let resolved = resolve_input_files(
        &[
            cwd.join("src/human.mts"),
            PathBuf::from("src/human.mts"),
            PathBuf::from("missing.mts"),
        ],
        &cwd,
        Path::new("/tmp"),
    );
    assert_eq!(resolved[0], cwd.join("src/human.mts"));
    assert_eq!(resolved[1], cwd.join("src/human.mts"));
    assert_eq!(resolved[2], PathBuf::from("/tmp/missing.mts"));

    for kind in [
        ExportKind::Class,
        ExportKind::Let,
        ExportKind::Var,
        ExportKind::TypeAlias,
        ExportKind::Interface,
    ] {
        assert!(!export_kind_str(&kind).is_empty());
    }

    assert_eq!(
        resolve_format(true, Some(Format::Human), true),
        Format::Json
    );
    assert_eq!(resolve_format(false, Some(Format::Md), true), Format::Md);
    assert_eq!(resolve_format(false, None, true), Format::Human);
    assert_eq!(resolve_format(false, None, false), Format::Json);
}
