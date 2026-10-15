use super::*;

#[test]
fn authoritative_and_pending_scope_receipts_preserve_static_and_dynamic_writes() {
    let catalog = Arc::new(PhaseCatalog {
        start: 10,
        end: 40,
        segments: vec![(10, 20, 0), (20, 30, 1), (30, 40, 2)],
    });
    let writes = [
        PhaseWrite {
            target: 100,
            name: "holder.box".into(),
            path: vec![Some("box".into())],
            phase: 1,
        },
        PhaseWrite {
            target: 101,
            name: "holder".into(),
            path: vec![None],
            phase: 2,
        },
    ];
    let mut histories = ReceiverHistories::default();
    histories.require_reference_proof(3, "holder");
    let scopes = [(100, 3)].into_iter().collect();
    let mut phases = ActiveReferencePhases::new(
        (&catalog, 50),
        &writes,
        Some(&scopes),
        &histories,
        None,
        true,
        Default::default(),
    );
    let key = (3, "holder".to_string());
    let read = |offset| Some((offset, None));
    assert_eq!(phases.capture_phase(Some(100)), Some((10, 1)));
    assert_eq!(phases.unresolved.get(&2), Some(&1));
    let cursor = phases.cursor(&histories, &key, None);
    assert!(cursor.changed("box", read(11), read(21)));
    assert!(!cursor.changed("sibling", read(11), read(21)));
    assert!(cursor.changed("sibling", read(21), read(31)));
    phases.resolve(101, 3, &histories);
    assert!(phases.pending.is_empty());
    assert!(phases.unresolved.is_empty());
    let cursor = phases.cursor(&histories, &key, None);
    assert!(cursor.changed("sibling", read(21), read(31)));
    assert!(!cursor.changed("sibling", read(11), read(21)));
    #[cfg(feature = "test-instrumentation")]
    {
        let mut facts = ValueFlowFacts::default();
        phases.record_counts(&mut facts);
        assert_eq!(facts.reference_phase_nodes, 2);
        assert_eq!(facts.reference_phase_write_entries, 2);
        assert_eq!(facts.reference_phase_pending_tokens, 0);
    }
    // A union changes the canonical component generation; old cursors fail closed.
    histories.link((3, "holder"), (3, "alias"));
    assert!(!phases.cursor(&histories, &key, None).allowed());
}

#[test]
fn missing_read_coordinates_and_foreign_owners_cannot_prove_identity() {
    let catalog = Arc::new(PhaseCatalog {
        start: 10,
        end: 30,
        segments: vec![(10, 30, 0)],
    });
    let mut histories = ReceiverHistories::default();
    histories.require_reference_proof(0, "holder");
    let phases = ActiveReferencePhases::new(
        (&catalog, 40),
        &[],
        None,
        &histories,
        None,
        true,
        Default::default(),
    );
    let key = (0, "holder".into());
    let cursor = phases.cursor(&histories, &key, None);
    let read = Some((15, None));
    assert!(!cursor.changed("box", read, read));
    assert!(cursor.changed("box", None, read));
    assert!(cursor.changed("box", read, None));
    assert!(cursor.changed("box", read, Some((15, Some(CallableId(7))))));
    assert!(cursor.changed("box", read, Some((50, None))));
    assert!(phases
        .cursor(&histories, &key, Some(CallableId(7)))
        .changed("box", read, read));
    let capture = identity_metadata::FreshCapture {
        origin: None,
        reads: Default::default(),
        epoch: 0,
        phase: None,
        completed_at: 15,
    };
    assert!(cursor.changed_after_capture("box", &capture, read));
}
