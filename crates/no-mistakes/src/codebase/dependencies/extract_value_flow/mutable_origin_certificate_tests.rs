use super::*;

#[test]
fn own_data_certificates_preserve_completion_and_empty_phase_projection() {
    let mut origins = ReceiverBindingOrigins::default();
    assert_eq!(
        origins.own_data_member(99, "missing", (21, None), "field"),
        None
    );
    let compact = origins.single_named_field("field", "client", Some(7), (8, None));
    // A first conditional write has no earlier alternative to preserve.
    origins.record((0, "box"), (20, None), compact, false);
    assert_eq!(origins.own_data_member(0, "box", (19, None), "field"), None);
    assert_eq!(
        origins.own_data_member(0, "box", (20, None), "field"),
        Some(20)
    );
    assert_eq!(origins.own_data_member(0, "box", (21, None), "other"), None);
    assert_eq!(
        origins.own_data_member(0, "box", (21, Some(CallableId(1))), "field"),
        None
    );
    origins
        .roots
        .get_mut(&0)
        .unwrap()
        .get_mut("box")
        .unwrap()
        .patterns = Some(Default::default());
    assert_eq!(
        origins.own_data_member(0, "box", (21, None), "field"),
        Some(20)
    );
    let key = (0, "box".to_string());
    assert!(origins
        .version(&key, (21, None))
        .unwrap()
        .selected
        .is_some());
    assert_eq!(
        origins.own_data_member(0, "missing", (21, None), "field"),
        None
    );
    let catalog = Arc::new(phase_catalog::PhaseCatalog {
        start: 20,
        end: 40,
        segments: vec![(20, 40, 1)],
    });
    origins
        .roots
        .get_mut(&0)
        .unwrap()
        .get_mut("box")
        .unwrap()
        .patterns
        .as_mut()
        .unwrap()
        .record(&catalog, 2, None, None, 0, true);
    // A phase before its first physical root write has no own-data certificate.
    assert_eq!(origins.own_data_member(0, "box", (25, None), "field"), None);
    origins
        .roots
        .get_mut(&0)
        .unwrap()
        .get_mut("box")
        .unwrap()
        .patterns = None;
    let full = ValueExpression::Object(
        Arc::new(FxHashMap::from_iter([(
            "field".into(),
            ValueExpression::Unknown,
        )])),
        false,
    );
    origins.record((0, "box"), (30, None), full, true);
    assert_eq!(
        origins.own_data_member(0, "box", (30, None), "field"),
        Some(30)
    );
    origins.record((0, "box"), (40, None), ValueExpression::Unknown, true);
    assert_eq!(origins.own_data_member(0, "box", (40, None), "field"), None);
    origins.record(
        (0, "box"),
        (50, Some(CallableId(1))),
        ValueExpression::Unknown,
        true,
    );
    assert_eq!(origins.own_data_member(0, "box", (60, None), "field"), None);
}

#[test]
fn untracked_roots_cannot_invent_projected_receiver_writes() {
    let origins = ReceiverBindingOrigins::default();
    let key = (99, "missing".into());
    assert!(origins.version(&key, (0, None)).is_none());
    let mut writes: Vec<nested::ReceiverWrite> = Vec::new();
    origins.append_references(
        &key,
        (&[], &[], (0, None)),
        &mut writes,
        0,
        RootVersion {
            context: None,
            selected: None,
            generation: 0,
        },
    );
    assert!(writes.is_empty());
}
