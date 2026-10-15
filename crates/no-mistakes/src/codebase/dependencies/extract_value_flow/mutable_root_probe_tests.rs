use super::*;

fn flow(source: &str) -> ValueFlowFacts {
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        false,
        false,
        true,
    )
    .value_flow
}
#[test]
fn existing_root_records_allocate_only_the_initial_owned_key() {
    let source = mutable_receiver_source("mutable-root-scale.mts");
    let facts = flow(&source);
    assert_eq!(facts.receiver_only_origins, 129);
    assert_eq!(facts.receiver_only_snapshot_tree_buffers, 0);
    assert_eq!(facts.receiver_only_owned_keys, 1);
    assert_eq!(facts.receiver_only_predecessor_queries, 0);
    assert!(
        facts.receiver_mutations.as_ref().unwrap()[&0]["client"].assignment_invalidated(
            &["get"],
            source.len() as u32,
            None,
            false
        )
    );
}
#[test]
fn captured_versions_bound_repeated_membership_probes() {
    let source = mutable_receiver_source("mutable-root-scale.mts");
    let facts = flow(&source);
    assert!(facts.receiver_only_membership_probes > 0);
    assert!(
        facts.receiver_only_membership_probes <= 6 * 128,
        "membership probes {}",
        facts.receiver_only_membership_probes
    );
    assert_eq!(facts.receiver_only_lookups, 128);
    assert_eq!(facts.receiver_only_snapshot_queries, 128);
}
#[test]
fn reused_versions_preserve_deferred_conditional_and_foreign_roots() {
    for name in [
        "mutable-let-pattern-lhs.mts",
        "mutable-let-pattern-static-rebind.mts",
        "mutable-let-conditional-root.mts",
        "mutable-let-foreign-root.mts",
    ] {
        let source = mutable_receiver_source(name);
        let facts = flow(&source);
        assert!(facts.receiver_only_membership_probes > 0, "{name}");
        assert!(facts.receiver_only_owned_keys > 0, "{name}");
        if name == "mutable-let-conditional-root.mts" {
            assert!(facts.receiver_only_predecessor_queries > 0);
        }
        assert!(
            facts.receiver_only_owned_keys <= facts.receiver_only_origins,
            "{name}"
        );
        assert!(
            facts
                .receiver_mutations
                .as_ref()
                .unwrap()
                .values()
                .flat_map(|names| names.values())
                .any(|history| history.assignment_invalidated(
                    &["get"],
                    source.len() as u32,
                    None,
                    false
                )),
            "{name}"
        );
    }
    let source = mutable_receiver_source("mutable-let-shadowed.mts");
    let facts = flow(&source);
    // Same spelling in separate lexical scopes still owns separate root keys.
    assert_eq!(facts.receiver_only_owned_keys, 2);
    assert_eq!(facts.receiver_only_origins, 2);
}
#[test]
fn ordinary_aliases_keep_the_root_index_cold() {
    let source = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/import-facts/fixture/many-member-writes.mts"),
    )
    .unwrap();
    let facts = flow(&source);
    assert_eq!(facts.receiver_only_origins, 0);
    assert_eq!(facts.receiver_only_owned_keys, 0);
    assert_eq!(facts.receiver_only_membership_probes, 0);
    assert!(
        facts.receiver_mutations.as_ref().unwrap()[&0]["primary"].assignment_invalidated(
            &["field255"],
            source.len() as u32,
            None,
            false
        )
    );
}

#[test]
fn saved_root_snapshot_insertion_orders() {
    for name in [
        "mutable-root-scale.mts",
        "mutable-let-pattern-lhs.mts",
        "mutable-let-pattern-static-rebind.mts",
        "mutable-let-conditional-root.mts",
        "mutable-let-foreign-root.mts",
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
    ] {
        let facts = flow(&mutable_receiver_source(name));
        println!(
            "SNAPSHOT_ORDER {name} {:?}",
            facts.receiver_only_insertion_order
        );
        assert!(facts.receiver_only_insertion_order.iter().sum::<usize>() > 0);
        if name == "mutable-root-scale.mts" {
            assert_eq!(facts.receiver_only_insertion_order, [129, 0, 0]);
        }
    }
}

#[test]
fn imported_terminal_identity_keeps_local_and_recovered_roots() {
    for name in [
        "mutable-import-local-recovery.mts",
        "mutable-import-var-recovery.mts",
        "mutable-import-local-shadow.mts",
        "mutable-import-namespace-shadow.mts",
    ] {
        let source = mutable_receiver_source(name);
        let facts = flow(&source);
        assert_eq!(facts.receiver_only_owned_keys, 1, "{name}");
        assert!(
            facts
                .receiver_mutations
                .as_ref()
                .and_then(|scopes| scopes.get(&0))
                .and_then(|names| names.get("replica"))
                .is_some_and(|history| history.assignment_invalidated(
                    &["get"],
                    source.len() as u32,
                    None,
                    false
                )),
            "{name}"
        );
    }
}
