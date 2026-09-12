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

#[test]
fn lang_route_domain_helpers_cover_glob_aliases_and_handler_shapes() {
    let interner = PathInterner::new();
    let root = Path::new("/repo");
    let mut options = super::GraphConfigOptions::default();
    assert!(super::route_file_allowed(
        root,
        Path::new("/repo/routes/users.py"),
        &options
    ));

    let mut builder = globset::GlobSetBuilder::new();
    builder.add(globset::Glob::new("routes/**").unwrap());
    options.project_route_globset = Some(builder.build().unwrap());
    assert!(super::route_file_allowed(
        root,
        Path::new("/repo/routes/users.py"),
        &options
    ));
    assert!(!super::route_file_allowed(
        root,
        Path::new("/repo/models/users.py"),
        &options
    ));
    assert!(super::route_file_allowed(
        root,
        Path::new("routes/users.py"),
        &options
    ));

    let file = LangFileFacts {
        path: PathBuf::from("/repo/routes/users.py"),
        package: Some("app".into()),
        module: Some("routes.users".into()),
        imports: vec![
            "Views=controllers.views.UserView".into(),
            "plain".into(),
            "controllers.views".into(),
        ],
        declarations: vec!["UserView".into()],
        route_handlers: vec![
            ("/users".into(), "routes.users.UserView".into()),
            ("/alias".into(), "Views".into()),
            ("/rails".into(), "admin/users#index".into()),
            ("/php".into(), "App\\Http\\Controllers\\UserController::index".into()),
        ],
        ..LangFileFacts::default()
    };
    let other = LangFileFacts {
        path: PathBuf::from("/repo/controllers/views.py"),
        package: Some("app".into()),
        module: Some("controllers.views".into()),
        declarations: vec!["UserView".into()],
        ..LangFileFacts::default()
    };
    let foreign = LangFileFacts {
        path: PathBuf::from("/repo/other/views.py"),
        package: Some("other".into()),
        module: Some("controllers.views".into()),
        declarations: vec!["UserView".into()],
        ..LangFileFacts::default()
    };
    let mut facts = facts_from(vec![file.clone(), other.clone(), foreign]);
    facts.files_by_module.insert(
        "routes.users.UserView".into(),
        BTreeSet::from([file.path.clone(), other.path.clone()]),
    );
    facts.declarations.insert(
        "UserView".into(),
        BTreeSet::from([other.path.clone()]),
    );

    let mut edges = Vec::new();
    super::emit_route_edges(root, &facts, &options, &mut edges, &interner);
    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == super::EdgeKind::RouteRef));

    assert_eq!(super::normalize_route_handler("'UsersView'.as_view()"), "UsersView");
    assert_eq!(
        super::route_handler_names("admin/users#index"),
        vec!["Admin::UsersController".to_string()]
    );
    assert_eq!(
        super::route_handler_names("users#show"),
        vec!["UsersController".to_string()]
    );
    assert_eq!(
        super::route_handler_names("App\\Http\\UserController::index"),
        vec![
            "UserController".to_string(),
            "App\\Http\\UserController".to_string()
        ]
    );
    let dotted = super::route_handler_names("pkg.views.user_view");
    assert!(dotted.contains(&"user_view".to_string()));
    assert!(dotted.contains(&"pkg.views".to_string()));
    assert_eq!(super::snake_to_pascal("user_view"), "UserView");
    assert_eq!(super::snake_to_pascal("_hidden_"), "Hidden");

    assert!(super::handler_module_matches("controllers.views.UserView", &other));
    assert!(super::handler_module_matches("App::UsersController", &file));
    let no_module = LangFileFacts {
        module: None,
        ..file.clone()
    };
    assert!(super::handler_module_matches("pkg.views.UserView", &no_module));
    assert!(super::handler_module_matches("routes.users.UserView", &file));

    assert_eq!(
        super::remap_aliased_handler(&file, "Views.index"),
        "controllers.views.UserView.index"
    );
    assert_eq!(super::remap_aliased_handler(&file, "plain"), "plain");
    assert_eq!(
        super::aliased_route_names(&file, "Views".into()),
        vec![
            "Views".to_string(),
            "controllers.views.UserView".to_string(),
            "UserView".to_string()
        ]
    );

    assert!(super::reference_target_allowed(&file, &file, "UserView"));
    assert!(super::reference_target_allowed(&file, &other, "controllers.views"));
    assert!(super::reference_target_allowed(
        &file,
        &other,
        "App::UsersController"
    ));
    assert!(super::import_reaches_module(
        "controllers.views",
        "controllers.views",
        "UserView"
    ));
    assert!(super::import_reaches_module(
        "pkg/views",
        "other/views",
        "UserView"
    ));
}
