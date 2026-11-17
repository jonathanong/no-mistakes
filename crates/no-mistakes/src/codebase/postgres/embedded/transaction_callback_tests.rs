use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions};

#[test]
fn configured_transaction_callback_parameters_are_scoped_transactions() {
    let path = crate::test_support::fixture_root("postgres-transaction-callbacks").join("query.ts");
    let source = std::fs::read_to_string(&path).expect("fixture");
    let mut options = EmbeddedSqlOptions::configured("", &["query".into()])
        .with_scoped_executors(&["transaction".into()], &[]);
    options.track_transaction_helpers = true;
    let calls = extract_embedded_sql_from_source(&path, &source, &options).calls;
    assert_eq!(calls.len(), 28, "{calls:?}");
    assert!(calls[0].executor_transaction_known, "{:?}", calls[0]);
    assert!(calls[0].transaction_id.is_some(), "{:?}", calls[0]);
    assert_eq!(calls[0].transaction_line, Some(4));
    assert_eq!(calls[1].transaction_id, None);
    assert_ne!(calls[2].transaction_id, calls[3].transaction_id);
    assert_eq!(calls[2].transaction_line, Some(11));
    assert_eq!(calls[3].transaction_line, Some(10));
    assert_eq!(calls[4].transaction_id, None);
    assert_eq!(calls[5].transaction_id, None);
    assert_eq!(calls[6].transaction_id, None);
    assert_eq!(calls[7].transaction_id, None);
    assert_eq!(calls[8].transaction_id, None);
    assert_eq!(calls[9].transaction_id, None);
    assert_eq!(calls[10].transaction_id, None);
    assert_eq!(calls[11].transaction_id, None);
    assert_eq!(calls[12].transaction_id, None);
    assert_eq!(calls[13].transaction_id, None);
    assert_eq!(calls[14].transaction_id, None);
    assert_eq!(calls[15].transaction_id, None);
    assert!(calls[16].transaction_id.is_some(), "{:?}", calls[16]);
    assert!(calls[16].executor_transaction_known, "{:?}", calls[16]);
    assert_eq!(calls[17].transaction_id, None);
    assert!(calls[18].transaction_id.is_some(), "{:?}", calls[18]);
    assert!(calls[18].executor_transaction_known, "{:?}", calls[18]);
    assert_eq!(calls[19].transaction_id, None);
    assert_eq!(calls[20].transaction_id, None);
    assert_eq!(calls[21].transaction_id, None);
    assert_eq!(calls[22].transaction_id, None);
    assert!(calls[23].transaction_id.is_some(), "{:?}", calls[23]);
    assert!(calls[23].executor_transaction_known);
    assert!(calls[24].executor_transaction_known, "{:?}", calls[24]);
    assert!(calls[24].transaction_id.is_some(), "{:?}", calls[24]);
    assert_eq!(calls[25].transaction_id, None);
    assert_eq!(calls[26].transaction_id, None);
    assert!(calls[27].executor_transaction_known, "{:?}", calls[27]);
    assert!(calls[27].transaction_id.is_some(), "{:?}", calls[27]);
}

#[test]
fn option_property_configuration_preserves_callable_transaction_aliases() {
    let path = crate::test_support::rule_fixture_root("postgres-lock-written-rows-only")
        .join("caller-options-followup/src/alias.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let mut options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["beginTransaction".into()], &[])
        .with_trusted_sql_tags(&[super::TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        }]);
    options.track_transaction_helpers = true;
    let without = extract_embedded_sql_from_source(&path, &source, &options).calls;
    let options = options.with_executor_option_properties(&["query".into()]);
    let with = extract_embedded_sql_from_source(&path, &source, &options).calls;
    assert_eq!(without.len(), 1, "{without:?}");
    assert_eq!(with.len(), 1, "{with:?}");
    assert_eq!(without[0].transaction_id, with[0].transaction_id);
    assert!(with[0].executor_transaction_known, "{with:?}");
    assert!(with[0].transaction_id.is_some(), "{with:?}");
    assert_eq!(without[0].sql_text, with[0].sql_text);
}
