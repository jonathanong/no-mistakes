use super::{route_fixture_source, *};
use std::collections::HashMap;

#[test]
fn summarizes_same_file_route_helper_export_aliases() {
    let source = route_fixture_source("route-helper-export-alias.ts");
    let facts = extract_route_ref_facts(&source, "entity-href.ts");
    let helpers = facts
        .route_helpers
        .iter()
        .map(|helper| (helper.name.as_str(), helper.patterns.clone()))
        .collect::<HashMap<_, _>>();

    assert_eq!(
        helpers.get("entityHref"),
        Some(&vec!["/aliased/*".to_string()])
    );
    assert_eq!(helpers.get("href"), Some(&vec!["/aliased/*".to_string()]));
}

#[test]
fn records_namespace_export_all_route_helper_imports() {
    let source = route_fixture_source("route-helper-namespace-export.ts");
    let facts = extract_route_ref_facts(&source, "links.ts");

    assert_eq!(facts.route_helper_imports.len(), 1);
    assert_eq!(facts.route_helper_imports[0].local, "links");
    assert_eq!(facts.route_helper_imports[0].imported, "*");
    assert_eq!(facts.route_helper_imports[0].source, "./entity-href");
}

#[test]
fn records_imported_route_helpers_reexported_as_default() {
    let source = route_fixture_source("route-helper-default-imported-alias.ts");
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
            ("default", "entityHref", "./entity-href"),
            ("entityHref", "entityHref", "./entity-href"),
        ]
    );
}
