use crate::codebase::postgres::statements::{SqlBoundItem, SqlQualifiedRead, SqlQualifiedScope};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundItemKind};

fn catalog() -> SchemaCatalog {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("view-key-evidence/schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn read() -> SqlQualifiedRead {
    SqlQualifiedRead {
        qualifier: "driver".into(),
        scopes: vec![SqlQualifiedScope {
            tables: vec!["base".into()],
            unknown: false,
        }],
    }
}

#[test]
fn region_lateral_drivers_need_exactly_one_preceding_owner() {
    let found = catalog();
    assert!(found.bounded_relation("base").is_some());
    assert!(super::qualified_dependencies(&[read()], &[], &found).is_none());
    let drivers = vec![
        SqlBoundItem::new(SqlBoundItemKind::Other, Some("driver".into()), (1, 1)),
        SqlBoundItem::new(SqlBoundItemKind::Other, Some("driver".into()), (1, 1)),
    ];
    assert!(super::qualified_dependencies(&[read()], &drivers, &found).is_none());
}
