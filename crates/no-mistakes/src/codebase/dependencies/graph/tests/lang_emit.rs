use crate::codebase::analysis_session::PathInterner;
use crate::codebase::lang_frontends::{LangFactMap, LangFileFacts};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn lang_file(path: &str, package: Option<&str>, module: Option<&str>) -> LangFileFacts {
    LangFileFacts {
        path: PathBuf::from(path),
        package: package.map(str::to_string),
        module: module.map(str::to_string),
        imports: vec!["dep".into()],
        declarations: vec!["Decl".into()],
        references: vec!["Decl".into()],
        mods: vec!["child".into()],
        ..LangFileFacts::default()
    }
}

fn facts_from(files: Vec<LangFileFacts>) -> LangFactMap {
    let mut facts = LangFactMap::default();
    for file in files {
        facts.index_file(file);
    }
    facts
}

#[test]
fn package_root_file_prefers_named_manifests_then_any_file() {
    let composer = BTreeSet::from([
        PathBuf::from("src/Service.php"),
        PathBuf::from("composer.json"),
    ]);
    assert_eq!(
        super::package_root_file(&composer).map(Path::as_os_str),
        Some(Path::new("composer.json").as_os_str())
    );

    let module = BTreeSet::from([
        PathBuf::from("src/lib/mod.rs"),
        PathBuf::from("src/util.rs"),
    ]);
    assert_eq!(
        super::package_root_file(&module).map(Path::as_os_str),
        Some(Path::new("src/lib/mod.rs").as_os_str())
    );

    let fallback = BTreeSet::from([PathBuf::from("src/util.rs")]);
    assert_eq!(
        super::package_root_file(&fallback).map(Path::as_os_str),
        Some(Path::new("src/util.rs").as_os_str())
    );
    assert!(super::package_root_file(&BTreeSet::new()).is_none());
}

#[test]
fn emit_helpers_cover_go_imports_mod_fallback_and_missing_packages() {
    let interner = PathInterner::new();
    let mut go = facts_from(vec![
        lang_file("/repo/a.go", Some("mod"), Some("a")),
        lang_file("/repo/dep.go", Some("mod"), Some("dep")),
    ]);
    go.files_by_module.insert(
        "dep".into(),
        BTreeSet::from([PathBuf::from("/repo/dep.go")]),
    );
    go.files
        .get_mut(&PathBuf::from("/repo/a.go"))
        .unwrap()
        .imports = vec!["dep".into()];
    let mut edges = Vec::new();
    super::emit_lang_edges(
        &go,
        super::EdgeKind::GoImport,
        super::EdgeKind::GoReference,
        &mut edges,
        &interner,
    );
    assert!(!edges.is_empty());

    let mut rust = facts_from(vec![
        lang_file("/repo/src/lib.rs", Some("crate"), None),
        lang_file("/repo/src/child.rs", Some("crate"), Some("child")),
    ]);
    edges.clear();
    super::emit_mod_edges(&rust, super::EdgeKind::RustMod, &mut edges, &interner);
    assert!(!edges.is_empty());

    rust.files_by_package
        .insert("empty".into(), BTreeSet::new());
    rust.package_path_deps
        .insert(("missing".into(), "crate".into()));
    rust.package_path_deps
        .insert(("empty".into(), "crate".into()));
    rust.package_path_deps
        .insert(("crate".into(), "missing".into()));
    rust.package_path_deps
        .insert(("crate".into(), "crate".into()));
    edges.clear();
    super::emit_package_edges(&rust, super::EdgeKind::RustPackage, &mut edges, &interner);
    super::emit_path_dep_package_edges(&rust, super::EdgeKind::RustPackage, &mut edges, &interner);
    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == super::EdgeKind::RustPackage));
}
