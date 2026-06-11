use super::*;

#[test]
fn route_helper_ref_patterns_stop_at_recursion_depth_limit() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let client = root.join("src/client.ts");
    let mut facts = TsFactMap::new();
    facts.insert(
        root.join("src/entity-href.ts"),
        TsFileFacts {
            route_helpers: vec![crate::codebase::ts_routes::refs::RouteHelper {
                name: "entityHref".to_string(),
                patterns: vec!["/prefix/*/suffix/*".to_string()],
            }],
            ..TsFileFacts::default()
        },
    );
    for index in 1..=6 {
        facts.insert(
            root.join(format!("src/depth{index}.ts")),
            TsFileFacts {
                route_helper_imports: vec![route_helper_import(
                    "entityHref",
                    "entityHref",
                    if index == 6 {
                        "./entity-href".to_string()
                    } else {
                        format!("./depth{}", index + 1)
                    },
                )],
                ..TsFileFacts::default()
            },
        );
    }
    facts.insert(
        root.join("src/namespace-depth6.ts"),
        TsFileFacts {
            route_helper_imports: vec![route_helper_import("links", "*", "./entity-href")],
            ..TsFileFacts::default()
        },
    );

    let deep_import = route_helper_import("entityHref", "entityHref", "./depth1");
    let boundary_import = route_helper_import("entityHref", "entityHref", "./depth6");
    let namespace_import = route_helper_import("links", "links", "./namespace-depth6");

    assert!(
        route_helper_patterns_from_import(&client, "entityHref", &deep_import, &facts, &resolver, 0)
            .is_none()
    );
    assert!(
        route_helper_patterns_from_import(
            &client,
            "entityHref",
            &boundary_import,
            &facts,
            &resolver,
            4,
        )
        .is_none()
    );
    assert_eq!(
        route_helper_patterns_from_import(
            &client,
            "entityHref",
            &boundary_import,
            &facts,
            &resolver,
            3,
        ),
        Some(vec!["/prefix/*/suffix/*".to_string()])
    );
    assert!(
        route_helper_namespace_member_patterns(
            &client,
            "links",
            "entityHref",
            &namespace_import,
            &facts,
            &resolver,
            4,
        )
        .is_none()
    );
    assert!(
        route_helper_namespace_member_patterns(
            &client,
            "links",
            "entityHref",
            &namespace_import,
            &facts,
            &resolver,
            5,
        )
        .is_none()
    );
    assert_eq!(
        route_helper_namespace_member_patterns(
            &client,
            "links",
            "entityHref",
            &namespace_import,
            &facts,
            &resolver,
            3,
        ),
        Some(vec!["/prefix/*/suffix/*".to_string()])
    );
}

fn route_helper_import(
    local: &str,
    imported: &str,
    source: impl Into<String>,
) -> crate::codebase::ts_routes::refs::RouteHelperImport {
    crate::codebase::ts_routes::refs::RouteHelperImport {
        local: local.to_string(),
        imported: imported.to_string(),
        source: source.into(),
    }
}
