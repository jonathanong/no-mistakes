use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundOutput, SqlBoundQuery};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundItemKind};

fn catalog() -> SchemaCatalog {
    let path =
        crate::test_support::rule_fixture_root("postgres-bounded-statements").join("schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn output(scalar: bool, source: Option<(usize, &str)>) -> SqlBoundOutput {
    SqlBoundOutput {
        name: None,
        caller_sized: false,
        scalar,
        source_column: source.map(|(index, column)| (index, column.to_owned())),
        array_aggregate_column: None,
        constant_null: false,
        recursive_domain: false,
    }
}

#[test]
fn region_scalar_output_uses_a_column_alias_and_rejects_a_missing_ordinal() {
    let found = catalog();
    let mut item = SqlBoundItem::new(
        SqlBoundItemKind::Query(SqlBoundQuery {
            outputs: vec![output(true, None)],
            ..SqlBoundQuery::default()
        }),
        None,
        (1, 1),
    );
    item.column_aliases = vec!["shown".into()];
    let query = SqlBoundQuery {
        items: vec![item],
        outputs: vec![output(false, Some((0, "shown")))],
        ..SqlBoundQuery::default()
    };
    assert!(super::scalar(&query, 0, &found));
    assert!(!super::scalar(&query, 1, &found));
    let missing = SqlBoundQuery {
        outputs: vec![output(false, Some((3, "id")))],
        ..SqlBoundQuery::default()
    };
    assert!(!super::single(&missing, &found));
}
