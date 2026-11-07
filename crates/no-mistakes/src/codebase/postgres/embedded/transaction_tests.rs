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
    let mut options = EmbeddedSqlOptions::configured("@example/db", &["query".into()])
        .with_scoped_executors(&["openTransaction".into()], &[])
        .with_executor_option_properties(&["query".into()]);
    options.track_transaction_helpers = true;
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

#[test]
fn sql_phase_two_projection_is_opt_in() {
    let path = super::tests::fixture("transaction-lock-profile.ts");
    let source = super::tests::read_fixture("transaction-lock-profile.ts");
    let mut options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["beginTransaction".into()], &[]);
    let file = extract_embedded_sql_from_source(&path, &source, &options);
    assert!(file.calls.iter().all(|call| !call.lock_scope_analysis));
    assert!(file.calls.iter().all(|call| call.lock_helper.is_none()));
    assert!(file.lock_helper_summaries.is_empty());
    assert!(file.imported_lock_helpers.is_empty());
    let statements = super::super::collect::dml::embedded_call_facts(&file, false, false);
    assert!(statements
        .iter()
        .all(|statement| statement.lock_writes.advisory_keys.is_empty()));
    options.track_transaction_helpers = true;
    let enabled = extract_embedded_sql_from_source(&path, &source, &options);
    assert!(enabled.calls.iter().all(|call| call.lock_scope_analysis));
    let statements = super::super::collect::dml::embedded_call_facts(&enabled, false, false);
    assert_eq!(
        statements
            .iter()
            .map(|statement| statement.lock_writes.advisory_keys.len())
            .sum::<usize>(),
        1
    );
}

#[test]
fn resource_transactions_preserve_legacy_extraction_without_lock_profile() {
    let extract = |name: &str, options: &EmbeddedSqlOptions| {
        extract_embedded_sql_from_source(
            &super::tests::fixture(name),
            &super::tests::read_fixture(name),
            options,
        )
    };
    let mut options = EmbeddedSqlOptions::configured("@example/db", &["query".into()])
        .with_scoped_executors(&["openTransaction".into()], &[]);
    let legacy = extract("transaction-resource-gate-legacy.ts", &options);
    let disabled = extract("transaction-resource-gate.ts", &options);
    let projection = |file: &super::EmbeddedSqlFileFacts| {
        file.calls
            .iter()
            .map(|call| {
                (
                    call.line,
                    call.sql_text.clone(),
                    call.kind,
                    call.executor_transaction_known,
                    call.transaction_line,
                    call.bound_parameters.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(disabled.executor_bindings, legacy.executor_bindings);
    assert_eq!(projection(&disabled), projection(&legacy));
    assert!(disabled
        .calls
        .iter()
        .filter(|call| [4, 6].contains(&call.line))
        .all(|call| call.transaction_id.is_none() && !call.executor_transaction_known));
    options.track_transaction_helpers = true;
    let enabled = extract("transaction-resource-gate.ts", &options);
    let resources: Vec<_> = enabled
        .calls
        .iter()
        .filter(|call| [4, 6].contains(&call.line))
        .collect();
    assert_eq!(resources.len(), 2);
    assert!(resources
        .iter()
        .all(|call| call.transaction_id.is_some() && call.executor_transaction_known));
    assert_ne!(resources[0].transaction_id, resources[1].transaction_id);
}
