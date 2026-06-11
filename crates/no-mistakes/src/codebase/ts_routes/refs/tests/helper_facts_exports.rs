use super::*;
use std::collections::HashMap;

#[test]
fn summarizes_same_file_route_helper_export_aliases() {
    let source = r#"
const entityHref = (entity: { id: string }) => `/aliased/${entity.id}`;
export { entityHref as href };
"#;
    let facts = extract_route_ref_facts(source, "entity-href.ts");
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
    let source = "export * as links from './entity-href';";
    let facts = extract_route_ref_facts(source, "links.ts");

    assert_eq!(facts.route_helper_imports.len(), 1);
    assert_eq!(facts.route_helper_imports[0].local, "links");
    assert_eq!(facts.route_helper_imports[0].imported, "*");
    assert_eq!(facts.route_helper_imports[0].source, "./entity-href");
}
