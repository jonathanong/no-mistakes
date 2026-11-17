use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions};

#[test]
fn configured_transaction_callback_parameters_are_scoped_transactions() {
    let path = super::tests::fixture("transaction-callback.ts");
    let source = super::tests::read_fixture("transaction-callback.ts");
    let mut options = EmbeddedSqlOptions::configured("", &["query".into()])
        .with_scoped_executors(&["transaction".into()], &[]);
    options.track_transaction_helpers = true;
    let calls = extract_embedded_sql_from_source(&path, &source, &options).calls;
    assert_eq!(calls.len(), 16, "{calls:?}");
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
}
