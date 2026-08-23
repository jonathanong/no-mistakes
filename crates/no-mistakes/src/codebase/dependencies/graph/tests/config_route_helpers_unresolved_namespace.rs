#[test]
fn route_helper_ref_patterns_ignore_unresolved_namespace_reexport_barrels() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let client = root.join("src/client.ts");
    let barrel = root.join("src/links.ts");
    let missing_file = root.join("src/missing-href.ts");
    let mut facts = TsFactMap::new();
    facts.insert(missing_file, TsFileFacts::default());
    facts.insert(barrel, TsFileFacts {
        route_helper_imports: vec![crate::codebase::ts_routes::refs::RouteHelperImport {
            local: "links".to_string(), imported: "*".to_string(), source: "./missing-href".to_string(),
        }], ..TsFileFacts::default()
    });
    let file_facts = TsFileFacts {
        route_helper_imports: vec![crate::codebase::ts_routes::refs::RouteHelperImport {
            local: "links".to_string(), imported: "links".to_string(), source: "./links".to_string(),
        }],
        route_helper_refs: vec![crate::codebase::ts_routes::refs::RouteHelperRef {
            callee: "links.entityHref".to_string(), wrapper_pattern: None, file: "src/client.ts".to_string(), line: 1,
        }], ..TsFileFacts::default()
    };
    assert!(route_helper_ref_patterns(&client, &file_facts, &facts, &resolver, &GraphFiles::from_files(
        facts.keys().cloned().chain(std::iter::once(client.clone())).collect(),
    )).is_empty());
}
