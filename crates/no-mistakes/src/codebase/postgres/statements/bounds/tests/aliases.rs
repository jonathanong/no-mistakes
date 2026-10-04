use super::facts;

#[test]
fn positional_alias_facts_cover_derived_and_function_items() {
    let all = facts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/derived-alias-facts.sql"
    )));
    assert_eq!(all.len(), 9);
    let aliases: Vec<Vec<&str>> = all
        .iter()
        .map(|fact| {
            fact.query
                .items
                .last()
                .unwrap()
                .column_aliases
                .iter()
                .map(String::as_str)
                .collect()
        })
        .collect();
    assert_eq!(
        aliases,
        [
            vec!["account_id", "email"],
            vec!["Account.ID", "Email"],
            vec![],
            vec![],
            vec!["account_id"],
            vec!["item"],
            vec!["account_id"],
            vec![],
            vec![],
        ]
    );
}
