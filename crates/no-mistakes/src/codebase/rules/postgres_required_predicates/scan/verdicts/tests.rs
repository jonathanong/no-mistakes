use crate::codebase::postgres::{SqlStatementFileFacts, SqlVerdictStatus};

fn compiled() -> super::super::super::CompiledOptions {
    super::super::super::compile_options(&super::super::super::Options {
        partition_keys: "require".to_string(),
        schema_catalog_path: "schema.json".to_string(),
        ..Default::default()
    })
    .unwrap()
}

#[test]
fn a_partition_key_without_a_catalog_abstains() {
    // Validation requires a path, but a caller can still reach judge without
    // the catalog. Guessing the key would be a wrong Known verdict.
    let judgment = super::judge(
        &super::Obligation::PartitionKey,
        &SqlStatementFileFacts::default(),
        &compiled(),
        None,
    );
    assert_eq!(judgment.status, SqlVerdictStatus::Abstain);
    assert_eq!(judgment.reason, Some("missing-relation-catalog"));
}

#[test]
fn indexed_probes_clear_each_verdict_obligation_but_keep_the_control() {
    use crate::codebase::postgres::statements::extract_sql_statement_facts_with_json_writes;
    use crate::codebase::postgres::SchemaCatalog;
    let root =
        crate::test_support::rule_fixture_root("postgres-required-predicates").join("index-probes");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-required-predicates/fixture/index-probes/constant.sql"
    ));
    let mut facts = extract_sql_statement_facts_with_json_writes(
        sql,
        false,
        None,
        crate::codebase::postgres::statements::SqlLocationDemand::None,
        false,
        true,
        false,
    );
    let control = facts.selects.pop().unwrap();
    for obligation in [
        super::Obligation::Text("topic_posts", "account_id IS NOT NULL"),
        super::Obligation::Column("topic_posts", "account_id"),
        super::Obligation::PartitionKey,
    ] {
        let judgment = super::judge(&obligation, &facts, &compiled(), Some(&catalog));
        assert_eq!(judgment.status, SqlVerdictStatus::ProvenClear);
    }
    facts.selects = vec![control];
    for obligation in [
        super::Obligation::Text("topic_posts", "account_id IS NOT NULL"),
        super::Obligation::Column("topic_posts", "account_id"),
        super::Obligation::PartitionKey,
    ] {
        let judgment = super::judge(&obligation, &facts, &compiled(), Some(&catalog));
        assert_eq!(judgment.status, SqlVerdictStatus::Violation);
    }
}
