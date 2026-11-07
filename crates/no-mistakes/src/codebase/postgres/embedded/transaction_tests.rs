use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions};

#[test]
fn scoped_transaction_and_bound_parameter_identity_are_shared() {
    let path = super::tests::fixture("transaction-membership.ts");
    let source = super::tests::read_fixture("transaction-membership.ts");
    let options = EmbeddedSqlOptions::configured("@example/db", &["query".into()])
        .with_scoped_executors(&["openTransaction".into()], &["TxExecutor".into()]);
    let calls = extract_embedded_sql_from_source(&path, &source, &options).calls;
    assert_eq!(calls.len(), 16);
    assert!(calls[0].transaction_id.is_some());
    assert_eq!(calls[0].transaction_line, Some(3));
    assert_eq!(calls[0].transaction_id, calls[1].transaction_id);
    assert_eq!(calls[0].bound_parameters, calls[1].bound_parameters);
    assert!(calls[0].bound_parameters[0].is_some());
    assert!(calls[2].executor_transaction_known);
    assert_eq!(calls[2].transaction_id, None);
    assert_ne!(calls[0].transaction_id, calls[3].transaction_id);
    assert!(!calls[4].executor_transaction_known);
    assert_eq!(calls[5].bound_parameters, [Some("literal:99".into())]);
    assert!(calls[6].bound_parameters[0].is_some());
    assert_eq!(calls[7].bound_parameters, [None]);
    assert!(!calls[8].executor_transaction_known);
    assert!(!calls[9].executor_transaction_known);
    assert_eq!(calls[9].transaction_id, calls[10].transaction_id);
    assert!(calls[10].executor_transaction_known);
    assert!(!calls[11].executor_transaction_known);
    assert!(!calls[12].executor_transaction_known);
    assert!(!calls[13].execution_unconditional);
    assert!(calls[14].execution_unconditional);
    assert!(calls[14].bound_parameters.is_empty());
    assert!(calls[15].executor_transaction_known);
    assert!(calls[15].transaction_id.is_none());
}

#[test]
fn transaction_metadata_survives_statement_variant_projection() {
    let path = super::tests::fixture("transaction-variants.ts");
    let source = super::tests::read_fixture("transaction-variants.ts");
    let options = EmbeddedSqlOptions::configured("@example/db", &["query".into()])
        .with_scoped_executors(&["openTransaction".into()], &[]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    let call = &facts.calls[0];
    assert_eq!(call.variants.len(), 2);
    assert!(call.transaction_id.is_some());
    let projected = call.statement_calls().collect::<Vec<_>>();
    assert_eq!(projected.len(), 2);
    for version in projected {
        assert_eq!(version.transaction_id, call.transaction_id);
        assert_eq!(version.transaction_line, call.transaction_line);
        assert_eq!(
            version.executor_transaction_known,
            call.executor_transaction_known
        );
        assert_eq!(
            version.execution_unconditional,
            call.execution_unconditional
        );
        assert_eq!(version.bound_parameters, call.bound_parameters);
    }
}

#[test]
fn resource_bindings_and_explicit_option_receivers_preserve_transaction_identity() {
    let path = super::tests::fixture("transaction-resource-options.ts");
    let source = super::tests::read_fixture("transaction-resource-options.ts");
    let options = EmbeddedSqlOptions::configured("@example/db", &["query".into()])
        .with_scoped_executors(&["openTransaction".into()], &[])
        .with_executor_option_properties(&["query".into()]);
    let calls = extract_embedded_sql_from_source(&path, &source, &options).calls;
    assert_eq!(calls.len(), 13, "{calls:?}");
    let outer = calls[0].transaction_id;
    assert!(outer.is_some());
    for index in [0, 1, 2, 5] {
        assert_eq!(calls[index].transaction_id, outer, "{index}");
        assert!(calls[index].executor_transaction_known, "{index}");
    }
    assert!(calls[3].transaction_id.is_some());
    assert_ne!(calls[3].transaction_id, outer);
    assert_eq!(calls[3].transaction_id, calls[4].transaction_id);
    for call in &calls[6..12] {
        assert!(!call.executor_transaction_known, "{call:?}");
        assert!(call.transaction_id.is_none());
    }
    assert!(calls[12].executor_transaction_known);
    assert!(calls[12].transaction_id.is_none());
}
