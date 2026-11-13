use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundQuery};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundItemKind};

fn catalog() -> SchemaCatalog {
    let path =
        crate::test_support::rule_fixture_root("postgres-bounded-statements").join("schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn opaque(alias: Option<&str>) -> SqlBoundItem {
    SqlBoundItem::new(SqlBoundItemKind::Opaque, alias.map(str::to_owned), (1, 1))
}

#[test]
fn region_recordset_roots_require_one_alternative_and_a_real_item() {
    let found = catalog();
    assert!(super::roots(&SqlBoundQuery::default(), 4, &[], &found).is_none());
    let mut item = SqlBoundItem::new(SqlBoundItemKind::Table("accounts".into()), None, (1, 1));
    item.pin_alternatives = 2;
    assert!(!super::explained(&item, &SqlBoundQuery::default(), &found));
    let inner = SqlBoundQuery {
        items: vec![
            opaque(Some("srf")),
            SqlBoundItem::new(SqlBoundItemKind::Table("accounts".into()), None, (1, 1)),
        ],
        ..SqlBoundQuery::default()
    };
    let outer = SqlBoundQuery {
        items: vec![
            SqlBoundItem::new(SqlBoundItemKind::Query(inner), None, (1, 1)),
            opaque(Some("side")),
        ],
        ..SqlBoundQuery::default()
    };
    assert!(super::recordset_carrier(&outer));
}
