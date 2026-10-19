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
