use super::*;
use crate::codebase::postgres::statements::facts::locations::work;

#[test]
fn json_key_walks_are_opt_in_and_preserve_existing_projections() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-json-relation-ids/fixture/db/fail.sql"
    ));
    work::begin();
    let without = extract_sql_statement_facts_with_json_writes(
        sql,
        false,
        None,
        SqlLocationDemand::Full,
        false,
    );
    assert_eq!(work::finish().json_key_walks, 0);
    assert!(without.json_writes.is_empty());
    work::begin();
    let mut with = extract_sql_statement_facts_with_json_writes(
        sql,
        false,
        None,
        SqlLocationDemand::Full,
        true,
    );
    assert!(work::finish().json_key_walks > 0);
    assert!(!with.json_writes.is_empty());
    with.json_writes.clear();
    with.variant_locations
        .as_mut()
        .unwrap()
        .positions
        .retain(|site, _| {
            !matches!(
                site,
                crate::codebase::postgres::statements::SqlFactSite::JsonWrite(_)
            )
        });
    assert_eq!(with, without);
}
