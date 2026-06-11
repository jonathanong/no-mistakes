use super::*;

#[test]
fn route_edge_push_skips_missing_pattern_map_entries() {
    let source = PathBuf::from("src/client.ts");
    let all_patterns = vec!["/prefix/:id".to_string()];
    let pattern_to_files = HashMap::new();
    let mut edges = Vec::new();

    push_matching_route_edges(
        &mut edges,
        &source,
        "/prefix/*",
        &all_patterns,
        &pattern_to_files,
    );

    assert!(edges.is_empty());
}

#[test]
fn route_pattern_skip_helper_covers_backend_filters() {
    let prefixes = vec!["/api/".to_string()];
    let exact = vec!["/healthz".to_string()];

    assert!(!route_pattern_should_skip(
        "/api/users",
        &prefixes,
        &exact,
        true,
        true
    ));
    assert!(!route_pattern_should_skip(
        "/healthz", &prefixes, &exact, true, true
    ));
    assert!(route_pattern_should_skip(
        "/frontend",
        &prefixes,
        &exact,
        true,
        true
    ));
    assert!(!route_pattern_should_skip(
        "/frontend",
        &prefixes,
        &exact,
        false,
        true
    ));
}

#[test]
fn route_helper_ref_patterns_cover_local_and_imported_variants() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let client = root.join("src/client.ts");
    let helper_file = root.join("src/entity-href.ts");

    let mut facts = TsFactMap::new();
    facts.insert(
        helper_file,
        TsFileFacts {
            route_helpers: vec![
                crate::codebase::ts_routes::refs::RouteHelper {
                    name: "entityHref".to_string(),
                    patterns: vec!["/prefix/*/suffix/*".to_string()],
                },
                crate::codebase::ts_routes::refs::RouteHelper {
                    name: "default".to_string(),
                    patterns: vec!["/prefix/*/suffix/default".to_string()],
                },
            ],
            ..TsFileFacts::default()
        },
    );
    let file_facts = TsFileFacts {
        route_helpers: vec![crate::codebase::ts_routes::refs::RouteHelper {
            name: "localHref".to_string(),
            patterns: vec!["/local/*".to_string()],
        }],
        route_helper_imports: vec![
            crate::codebase::ts_routes::refs::RouteHelperImport {
                local: "entityHref".to_string(),
                imported: "entityHref".to_string(),
                source: "./entity-href".to_string(),
            },
            crate::codebase::ts_routes::refs::RouteHelperImport {
                local: "defaultEntityHref".to_string(),
                imported: "default".to_string(),
                source: "./entity-href".to_string(),
            },
            crate::codebase::ts_routes::refs::RouteHelperImport {
                local: "links".to_string(),
                imported: "*".to_string(),
                source: "./entity-href".to_string(),
            },
        ],
        route_helper_refs: vec![
            crate::codebase::ts_routes::refs::RouteHelperRef {
                callee: "localHref".to_string(),
                file: "src/client.ts".to_string(),
                line: 1,
            },
            crate::codebase::ts_routes::refs::RouteHelperRef {
                callee: "entityHref".to_string(),
                file: "src/client.ts".to_string(),
                line: 2,
            },
            crate::codebase::ts_routes::refs::RouteHelperRef {
                callee: "localHref".to_string(),
                file: "src/client.ts".to_string(),
                line: 3,
            },
            crate::codebase::ts_routes::refs::RouteHelperRef {
                callee: "entityHref".to_string(),
                file: "src/client.ts".to_string(),
                line: 4,
            },
            crate::codebase::ts_routes::refs::RouteHelperRef {
                callee: "defaultEntityHref".to_string(),
                file: "src/client.ts".to_string(),
                line: 5,
            },
            crate::codebase::ts_routes::refs::RouteHelperRef {
                callee: "links.entityHref".to_string(),
                file: "src/client.ts".to_string(),
                line: 6,
            },
            crate::codebase::ts_routes::refs::RouteHelperRef {
                callee: "missing.entityHref".to_string(),
                file: "src/client.ts".to_string(),
                line: 7,
            },
        ],
        ..TsFileFacts::default()
    };

    assert_eq!(
        route_helper_ref_patterns(&client, &file_facts, &facts, &resolver),
        vec![
            "/local/*".to_string(),
            "/prefix/*/suffix/*".to_string(),
            "/prefix/*/suffix/default".to_string(),
        ]
    );
}
