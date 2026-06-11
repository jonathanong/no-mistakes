use super::{route_fixture_source, *};

#[test]
fn records_all_helper_calls_inside_route_context_expressions() {
    let source = route_fixture_source("route-helper-contexts.tsx");
    let facts = extract_route_ref_facts(&source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec![
            "aHref", "bHref", "aHref", "bHref", "aHref", "bHref", "aHref", "bHref", "aHref",
            "bHref", "aHref", "bHref",
        ]
    );
}

#[test]
fn ignores_shadowed_helper_calls_in_route_contexts() {
    let source = route_fixture_source("route-helper-shadowed-contexts.tsx");
    let facts = extract_route_ref_facts(&source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["entityHref", "links.entityHref"]
    );
}

#[test]
fn records_local_aliases_and_namespace_named_imports_in_route_contexts() {
    let source = route_fixture_source("route-helper-alias-contexts.tsx");
    let facts = extract_route_ref_facts(&source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["entityHref", "links.entityHref", "entityHref"]
    );
}

#[test]
fn ignores_loop_and_switch_bindings_that_shadow_route_helpers() {
    let source = route_fixture_source("route-helper-loop-switch-shadowing.tsx");
    let facts = extract_route_ref_facts(&source, "component.tsx");
    assert_eq!(
        facts
            .route_helper_refs
            .iter()
            .map(|route_ref| route_ref.callee.as_str())
            .collect::<Vec<_>>(),
        vec!["entityHref", "entityHref"]
    );
}
