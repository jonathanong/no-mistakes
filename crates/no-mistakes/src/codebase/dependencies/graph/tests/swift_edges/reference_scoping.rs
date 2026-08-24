use super::*;

#[test]
fn swift_references_do_not_select_duplicate_symbols_from_unrelated_packages() {
    let source = p("Client/App/Sources/App/App.swift");
    let declared = p("Client/Core/Sources/Core/Client.swift");
    let unrelated = p("Other/Core/Sources/Core/Client.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts.files.insert(source.clone(), swift_file(&source, Some("App"), vec!["Client"]));
    facts.files.insert(declared.clone(), swift_file(&declared, Some("Core"), Vec::new()));
    facts.files.insert(unrelated.clone(), swift_file(&unrelated, Some("Core"), Vec::new()));
    facts.declarations.insert("Client".to_string(), BTreeSet::from([declared.clone(), unrelated.clone()]));
    facts.packages.extend([app_package(), crate::codebase::swift::SwiftPackageFacts { package_root: p("Client/Core"), ..Default::default() }, crate::codebase::swift::SwiftPackageFacts { package_root: p("Other/Core"), ..Default::default() }]);
    let mut edges = Vec::new();
    collect_swift_reference_edges(&facts, &mut edges, &crate::codebase::analysis_session::PathInterner::new());
    assert!(edges.contains(&(NodeId::file(source), NodeId::file(declared), EdgeKind::SwiftReference)));
    assert!(!edges.iter().any(|(_, target, kind)| *kind == EdgeKind::SwiftReference && target.as_file() == Some(&unrelated)));
}

fn swift_file(path: &PathBuf, target: Option<&str>, references: Vec<&str>) -> crate::codebase::swift::SwiftFileFacts {
    crate::codebase::swift::SwiftFileFacts { path: path.clone(), target: target.map(str::to_string), references: references.into_iter().map(str::to_string).collect(), ..Default::default() }
}

fn app_package() -> crate::codebase::swift::SwiftPackageFacts {
    crate::codebase::swift::SwiftPackageFacts { package_root: p("Client/App"), local_package_paths: vec!["../Core".to_string()], local_package_bindings: BTreeMap::from([("../Core".to_string(), "core".to_string())]), targets: BTreeMap::from([("App".to_string(), crate::codebase::swift::SwiftTargetFacts { name: "App".to_string(), product_packages: BTreeMap::from([("Core".to_string(), "core".to_string())]), dependencies: vec!["Core".to_string()], ..Default::default() })]), ..Default::default() }
}
