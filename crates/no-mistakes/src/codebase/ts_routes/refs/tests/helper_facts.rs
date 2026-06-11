use super::{super::*, route_fixture_source};
use std::collections::HashMap;

#[test]
fn summarizes_basic_route_helper_patterns() {
    let source = route_fixture_source("route-helper-basic.ts");
    let facts = extract_route_ref_facts(&source, "links.ts");
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(facts.route_helpers[0].name, "entityHref");
    assert_eq!(facts.route_helpers[0].patterns, vec!["/prefix/*/suffix/*"]);
}

#[test]
fn summarizes_default_exported_route_helper_patterns() {
    let named = route_fixture_source("route-helper-default-named.ts");
    let facts = extract_route_ref_facts(&named, "entity-href.ts");
    let helpers = facts
        .route_helpers
        .iter()
        .map(|helper| (helper.name.as_str(), helper.patterns.clone()))
        .collect::<HashMap<_, _>>();
    assert_eq!(
        helpers.get("default"),
        Some(&vec!["/entities/*".to_string()])
    );
    assert_eq!(
        helpers.get("entityHref"),
        Some(&vec!["/entities/*".to_string()])
    );

    let anonymous = route_fixture_source("route-helper-default-anonymous.ts");
    let facts = extract_route_ref_facts(&anonymous, "anonymous-href.ts");
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(facts.route_helpers[0].name, "default");
    assert_eq!(facts.route_helpers[0].patterns, vec!["/anonymous/*"]);

    let function_expression = route_fixture_source("route-helper-default-function-expression.ts");
    let facts = extract_route_ref_facts(&function_expression, "function-expression-href.ts");
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(facts.route_helpers[0].name, "default");
    assert_eq!(
        facts.route_helpers[0].patterns,
        vec!["/function-expression/*"]
    );

    let declaration = route_fixture_source("route-helper-default-declaration.ts");
    let facts = extract_route_ref_facts(&declaration, "declaration-href.ts");
    assert!(facts.route_helpers.is_empty());

    let parenthesized_function_expression =
        route_fixture_source("route-helper-default-parenthesized-function.ts");
    let facts = extract_route_ref_facts(
        &parenthesized_function_expression,
        "parenthesized-function-href.ts",
    );
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(facts.route_helpers[0].name, "default");
    assert_eq!(
        facts.route_helpers[0].patterns,
        vec!["/parenthesized-function/*"]
    );

    let nested_parenthesized_function_expression =
        route_fixture_source("route-helper-default-nested-parenthesized-function.ts");
    let facts = extract_route_ref_facts(
        &nested_parenthesized_function_expression,
        "nested-parenthesized-function-href.ts",
    );
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(
        facts.route_helpers[0].patterns,
        vec!["/nested-parenthesized-function/*"]
    );

    let non_helper_expression = route_fixture_source("route-helper-default-non-helper.ts");
    let facts = extract_route_ref_facts(&non_helper_expression, "non-helper-expression.ts");
    assert!(facts.route_helpers.is_empty());

    let default_alias = route_fixture_source("route-helper-default-alias.ts");
    let facts = extract_route_ref_facts(&default_alias, "default-alias-href.ts");
    let helpers = facts
        .route_helpers
        .iter()
        .map(|helper| (helper.name.as_str(), helper.patterns.clone()))
        .collect::<HashMap<_, _>>();
    assert_eq!(
        helpers.get("default"),
        Some(&vec!["/aliased-default/*".to_string()])
    );
    assert_eq!(
        helpers.get("entityHref"),
        Some(&vec!["/aliased-default/*".to_string()])
    );
}

#[test]
fn summarizes_nested_route_helpers_with_suffixes() {
    let source = route_fixture_source("route-helper-nested-suffixes.ts");
    let facts = extract_route_ref_facts(&source, "entity-href.ts");
    let helper = |name: &str| {
        facts
            .route_helpers
            .iter()
            .find(|helper| helper.name == name)
            .map(|helper| helper.patterns.clone())
            .unwrap_or_default()
    };
    assert_eq!(helper("createTopicPathname"), vec!["/*/*"]);
    assert_eq!(helper("topicTagsHref"), vec!["/*/*/tags/*"]);
    assert_eq!(helper("topicHref"), vec!["/*/*", "/*/*/*"]);
}

#[test]
fn summarizes_route_helper_edge_expression_shapes() {
    let source = route_fixture_source("route-helper-edge-shapes.ts");
    let facts = extract_route_ref_facts(&source, "edge-shapes.ts");
    let helper = |name: &str| {
        facts
            .route_helpers
            .iter()
            .find(|helper| helper.name == name)
            .map(|helper| helper.patterns.clone())
            .unwrap_or_default()
    };
    assert_eq!(helper("logicalHref"), vec!["/logical/*"]);
    assert_eq!(helper("assertedHref"), vec!["/asserted/*"]);
    assert_eq!(helper("angleAssertedHref"), vec!["/angle/*"]);
    assert_eq!(helper("wrappedObjectHref"), vec!["/object/*"]);
    assert!(helper("missingReturnHref").is_empty());
    let capped_patterns = helper("cappedHref");
    assert_eq!(capped_patterns.len(), 16);
    assert!(capped_patterns.contains(&"/a/c/e/g/i".to_string()));
    assert!(capped_patterns.contains(&"/a/d/f/h/j".to_string()));
    assert!(!capped_patterns.contains(&"/z/y/x/w/v".to_string()));
    assert_eq!(helper("branchedHref"), vec!["/active/*", "/archive/*"]);
    assert_eq!(
        helper("localBranchHref"),
        vec!["/active-local/*", "/archive-local/*"]
    );
    assert_eq!(helper("deadReturnHref"), vec!["/dead-live/*"]);
    assert_eq!(
        helper("nestedStatementHref"),
        vec![
            "/nested-block/*",
            "/nested-catch/*",
            "/nested-fallback/*",
            "/nested-switch/*",
            "/nested-try/*"
        ]
    );
    assert_eq!(helper("topLevelBlockReturnHref"), vec!["/block-return/*"]);
    assert_eq!(
        helper("topLevelBlockAssignHref"),
        vec!["/block-assign/*/details"]
    );
    assert_eq!(
        helper("reassignedHref"),
        vec!["/users/*", "/users/*/tabs/*"]
    );
    assert_eq!(
        helper("assignedHref"),
        vec!["/assigned/*", "/assigned/*/tabs/*"]
    );
    assert_eq!(helper("topLevelAssignedHref"), vec!["/top/*/edit"]);
    assert_eq!(
        helper("memberAssignmentIgnoredHref"),
        vec!["/member-assignment/*"]
    );
    assert_eq!(helper("destructuredLocalHref"), vec!["/destructured/*"]);
    assert_eq!(
        helper("reassignedBranchHref"),
        vec!["/items/*/a", "/items/*/b"]
    );
    assert_eq!(
        helper("switchHref"),
        vec!["/orgs/*", "/unknown/*", "/users/*"]
    );
    assert_eq!(
        helper("reassignedSwitchHref"),
        vec!["/switch/*", "/switch/*/settings"]
    );
    assert_eq!(
        helper("switchAllBranchesAssignedHref"),
        vec!["/switch-all/*/default", "/switch-all/*/settings"]
    );
    assert_eq!(
        helper("switchFallthroughHref"),
        vec![
            "/fallthrough/*",
            "/fallthrough/*/a",
            "/fallthrough/*/a/b",
            "/fallthrough/*/b"
        ]
    );
    assert_eq!(helper("emptySwitchHref"), vec!["/empty-switch/*"]);
    assert_eq!(helper("tryHref"), vec!["/fallback/*", "/try/*"]);
    assert_eq!(helper("tryFinallyHref"), vec!["/finally/*"]);
    assert!(helper("catchParamShadowHref").is_empty());
    assert_eq!(helper("urlObjectHref"), vec!["/object/*"]);
    assert_eq!(helper("spreadObjectHref"), vec!["/spread/*"]);
}

#[test]
fn summarizes_deep_route_helper_calls_as_wildcards() {
    let source = route_fixture_source("route-helper-deep.ts");
    let facts = extract_route_ref_facts(&source, "deep-href.ts");
    let helper = facts
        .route_helpers
        .iter()
        .find(|helper| helper.name == "deepHref")
        .expect("deep helper should be summarized");
    assert_eq!(helper.patterns, vec!["/deep/*"]);
}

#[test]
fn records_helper_calls_only_in_route_contexts() {
    let source = route_fixture_source("route-helper-route-contexts.tsx");
    let facts = extract_route_ref_facts(&source, "component.tsx");
    assert_eq!(facts.route_helper_refs.len(), 37);
    assert!(facts
        .route_helper_refs
        .iter()
        .all(|route_ref| route_ref.callee == "entityHref"));
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .filter_map(|route_ref| route_ref.wrapper_pattern.as_deref())
            .collect::<Vec<_>>(),
        vec![
            "/admin{route_helper}",
            "{route_helper}/settings",
            "/admin/{route_helper}/settings",
            "/admin{route_helper}",
        ]
    );
}

#[test]
fn records_same_file_exported_route_helper_calls() {
    let source = route_fixture_source("route-helper-exported-local-ref.tsx");
    let facts = extract_route_ref_facts(&source, "component.tsx");
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(facts.route_helper_refs.len(), 1);
    assert_eq!(facts.route_helper_refs[0].callee, "entityHref");
}

#[test]
fn records_same_file_exported_const_route_helper_calls() {
    let source = route_fixture_source("route-helper-exported-local-const-ref.tsx");
    let facts = extract_route_ref_facts(&source, "component.tsx");
    assert_eq!(facts.route_helpers.len(), 1);
    assert_eq!(facts.route_helper_refs.len(), 1);
    assert_eq!(facts.route_helper_refs[0].callee, "entityHref");
}

#[test]
fn records_named_reexport_route_helper_imports() {
    let source = route_fixture_source("route-helper-named-reexport.ts");
    let facts = extract_route_ref_facts(&source, "links.ts");
    assert_eq!(
        facts
            .route_helper_imports
            .iter()
            .map(|import| {
                (
                    import.local.as_str(),
                    import.imported.as_str(),
                    import.source.as_str(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            ("entityHref", "entityHref", "./entity-href"),
            ("renamedHref", "otherHref", "./entity-href"),
        ]
    );
}

#[test]
fn records_star_reexport_route_helper_imports() {
    let source = route_fixture_source("route-helper-star-reexport.ts");
    let facts = extract_route_ref_facts(&source, "links.ts");
    assert_eq!(
        facts
            .route_helper_imports
            .iter()
            .map(|import| {
                (
                    import.local.as_str(),
                    import.imported.as_str(),
                    import.source.as_str(),
                )
            })
            .collect::<Vec<_>>(),
        vec![("*", "*", "./entity-href")]
    );
}

#[test]
fn sorts_route_helper_imports_and_refs_deterministically() {
    let source = route_fixture_source("route-helper-import-sort.tsx");
    let facts = extract_route_ref_facts(&source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_imports
            .iter()
            .map(|import| {
                (
                    import.local.as_str(),
                    import.imported.as_str(),
                    import.source.as_str(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            ("alphaHref", "alphaHref", "./a"),
            ("betaHref", "betaHref", "./b"),
            ("entityHref", "entityHref", "./entity-href"),
        ]
    );
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| (route_ref.line, route_ref.callee.as_str()))
            .collect::<Vec<_>>(),
        vec![(5, "alphaHref"), (5, "betaHref"), (6, "entityHref")]
    );
}

#[test]
fn records_route_helper_calls_inside_url_wrappers() {
    let source = route_fixture_source("route-helper-url-wrappers.tsx");
    let facts = extract_route_ref_facts(&source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["entityHref", "entityHref", "entityHref", "entityHref"]
    );
}

#[test]
fn records_route_helper_calls_inside_type_wrappers() {
    let source = route_fixture_source("route-helper-type-wrappers.ts");
    let facts = extract_route_ref_facts(&source, "component.ts");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["entityHref", "entityHref", "entityHref"]
    );
}

#[test]
fn records_namespace_helper_calls_in_route_contexts() {
    let source = route_fixture_source("route-helper-namespace-context.tsx");
    let facts = extract_route_ref_facts(&source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["links.entityHref", "links.topicHref"]
    );
    assert_eq!(facts.route_helper_imports[0].imported, "*");
}

fn with_route_fixture_var_init<T>(
    name: &str,
    index: usize,
    analyze: impl for<'a> FnOnce(&'a Expression<'a>, &'a Program<'a>) -> T,
) -> T {
    let allocator = Allocator::default();
    let source = route_fixture_source(name);
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let Statement::VariableDeclaration(var_decl) = &ret.program.body[index] else {
        panic!("expected variable declaration");
    };
    let expr = var_decl.declarations[0]
        .init
        .as_ref()
        .expect("expected initializer");
    analyze(expr, &ret.program)
}

#[test]
fn recognizes_optional_member_expressions_as_route_contexts() {
    with_route_fixture_var_init(
        "route-helper-optional-route-context.ts",
        1,
        |expr, program| {
            let mut bindings = collect_import_bindings(&program.body);
            collect_router_bindings_for_scope(&program.body, &mut bindings);
            assert!(callee_is_route_context(expr, &bindings));
        },
    );
}

#[test]
fn recognizes_optional_call_expressions_as_route_contexts() {
    with_route_fixture_var_init(
        "route-helper-optional-route-context.ts",
        2,
        |expr, program| {
            let mut bindings = collect_import_bindings(&program.body);
            collect_router_bindings_for_scope(&program.body, &mut bindings);
            assert!(callee_is_route_context(expr, &bindings));
        },
    );
}

#[test]
fn recognizes_optional_member_expressions_as_helper_callees() {
    with_route_fixture_var_init("route-helper-optional-helper-callee.ts", 0, |expr, _| {
        assert_eq!(
            route_helper_callee_name_from_callee(expr),
            Some("links.entityHref".to_string())
        );
    });
}

#[test]
fn recognizes_optional_call_expressions_as_helper_callees() {
    with_route_fixture_var_init("route-helper-optional-helper-callee.ts", 1, |expr, _| {
        assert_eq!(
            route_helper_callee_name_from_callee(expr),
            Some("links.entityHref".to_string())
        );
    });
}

#[test]
fn extracts_route_refs_from_existing_program() {
    let allocator = Allocator::default();
    let source = route_fixture_source("route-helper-program-route.ts");
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let refs = extract_route_refs_from_program(&ret.program, &source, "program.ts");

    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].pattern, "/program-route");
}
