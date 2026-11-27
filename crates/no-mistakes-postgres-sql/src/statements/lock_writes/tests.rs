use super::collect;

fn statements(name: &str) -> Vec<sqlparser::ast::Statement> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/statements")
        .join(name);
    crate::parse::parse_postgres_sql(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn lock_write_lifecycle_records_executed_targets_and_invalidates_join_keys() {
    let statements = statements("lock-write-lifecycle.sql");
    let legacy = collect::<false>(&statements, None, false, None, &[]);
    let advanced = collect::<true>(&statements, None, false, None, &[]);
    let mut shared = advanced.clone();
    assert_eq!(
        std::mem::take(&mut shared.ddl_events),
        [super::DdlEvent::Lock(0), super::DdlEvent::Lock(1)]
    );
    assert_eq!(legacy, shared);
    assert!(advanced.relation_identity_changed);
    assert!(advanced.transaction_boundary);
    assert_eq!(
        advanced.table_locks,
        [Some("accounts".into()), Some("sources".into())]
    );
    assert_eq!(advanced.writes.len(), 4);
    for row in &advanced.writes {
        assert_eq!(row.relation.as_deref(), Some("accounts"));
        assert!(row.has_predicate);
    }
    for index in [0, 2] {
        assert_eq!(advanced.writes[index].key, Some(("id".into(), "$1".into())));
    }
    for index in [1, 3] {
        assert_eq!(advanced.writes[index].key, None);
    }
}

#[test]
fn cte_lock_scopes_do_not_leak_into_following_statements() {
    let statements = statements("lock-cte-scopes.sql");
    for facts in [
        collect::<false>(&statements, None, false, None, &[]),
        collect::<true>(&statements, None, false, None, &[]),
    ] {
        assert_eq!(facts.locks.len(), 2);
        assert_eq!(facts.locks[0].relation, None);
        assert_eq!(facts.locks[1].relation.as_deref(), Some("accounts"));
        assert_eq!(facts.locks[1].key, Some(("id".into(), "$1".into())));
        assert!(!facts.transaction_boundary);
        assert!(!facts.relation_identity_changed);
    }
}

#[test]
fn ambiguous_delete_targets_do_not_supply_trigger_evidence() {
    let statements = statements("trigger-write-boundaries.sql");
    let deletion = collect::<true>(&statements[..1], None, false, None, &[]);
    assert!(deletion.trigger_writes.is_empty());
    for statement in &statements[1..] {
        let facts = collect::<true>(std::slice::from_ref(statement), None, false, None, &[]);
        assert!(facts.dynamic_setting_name);
    }
}

#[test]
fn prelock_edges_reject_non_plain_targets_and_record_reversed_and_keys() {
    let statements = statements("prelock-edges.sql");
    for (index, statement) in statements.iter().enumerate() {
        let facts = collect::<true>(std::slice::from_ref(statement), None, true, None, &[]);
        match index {
            0..=3 | 5 | 6 => {
                assert!(facts.prelock_attempt, "index {index}");
                assert!(facts.prelock.is_none(), "index {index}");
            }
            4 => {
                let (row, only) = facts.prelock.expect("ONLY prelock");
                assert!(only);
                assert_eq!(row.relation.as_deref(), Some("accounts"));
                assert_eq!(row.key, Some(("id".into(), "$1".into())));
            }
            7 => {
                assert_eq!(facts.conditional_write_keys.len(), 2);
                assert_eq!(
                    facts.conditional_write_keys[0].key,
                    Some(("id".into(), "$1".into()))
                );
                assert_eq!(
                    facts.conditional_write_keys[1].key,
                    Some(("tenant_id".into(), "$2".into()))
                );
            }
            8..=13 => {
                assert!(facts.conditional_write_keys.is_empty(), "index {index}");
            }
            14 | 15 => {
                assert!(facts.linked_cte_locks.is_empty(), "index {index}");
            }
            16 => {
                assert_eq!(
                    facts.linked_cte_locks.get(&0),
                    Some(&super::CteWriteLink::RequiresNonNull("id".into()))
                );
            }
            _ => unreachable!(),
        }
    }
}
