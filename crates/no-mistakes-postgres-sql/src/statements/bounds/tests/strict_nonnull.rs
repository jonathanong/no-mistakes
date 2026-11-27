use crate::statements::SqlBoundItemKind;

fn columns(sql: &str) -> Vec<(String, Vec<String>, Vec<String>)> {
    super::facts(sql)
        .into_iter()
        .flat_map(|fact| fact.query.items)
        .map(|item| {
            let kind = match &item.kind {
                SqlBoundItemKind::Table(name) => format!("table {name}"),
                SqlBoundItemKind::Query(_) => "query".to_string(),
                SqlBoundItemKind::Other => "other".to_string(),
                SqlBoundItemKind::Opaque => "opaque".to_string(),
            };
            (kind, item.nonnull_columns, item.strict_nonnull_columns)
        })
        .collect()
}

#[test]
fn repeated_lower_records_the_base_column_once() {
    assert_eq!(
        columns("SELECT 1 FROM accounts WHERE lower(name) = $1 AND lower(name) = $2"),
        vec![(
            "table accounts".to_string(),
            vec![],
            vec!["name".to_string()]
        )]
    );
}

#[test]
fn outer_join_on_skips_the_preserved_table() {
    assert_eq!(
        columns(
            "SELECT 1 FROM accounts a LEFT JOIN tenants t ON lower(a.name) = lower(t.slug) WHERE lower(t.slug) = $1",
        ),
        vec![
            ("table accounts".to_string(), vec![], vec![]),
            (
                "table tenants".to_string(),
                vec![],
                vec!["slug".to_string()]
            ),
        ]
    );
}

#[test]
fn inner_join_on_still_records_the_strict_column() {
    assert_eq!(
        columns("SELECT 1 FROM accounts a INNER JOIN tenants t ON lower(a.name) = t.id"),
        vec![
            (
                "table accounts".to_string(),
                vec![],
                vec!["name".to_string()]
            ),
            ("table tenants".to_string(), vec!["id".to_string()], vec![]),
        ]
    );
}

#[test]
fn a_derived_item_does_not_record_a_strict_base_column() {
    let found = columns("SELECT 1 FROM (VALUES ('a')) AS s(name) WHERE lower(name) = 'a'");
    assert!(
        found.iter().any(|(kind, _, _)| kind == "query"),
        "{found:?}"
    );
    assert!(
        found.iter().all(|(_, _, strict)| strict.is_empty()),
        "{found:?}"
    );
}
