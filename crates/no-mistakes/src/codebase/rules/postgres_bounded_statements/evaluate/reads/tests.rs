use super::ViewBoundMode;
use crate::codebase::postgres::SchemaCatalog;
use std::collections::BTreeMap;

fn catalog() -> SchemaCatalog {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("view-key-evidence/schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn region_a_bounded_view_contract_is_not_an_offender() {
    let found = catalog();
    let modes = BTreeMap::from([("aliased".to_owned(), ViewBoundMode::Bounded)]);
    let views = super::ViewBounds::compile(&modes, &found).unwrap();
    assert!(super::relation_offender("aliased", 1, 1, &found, &views).is_none());
}
