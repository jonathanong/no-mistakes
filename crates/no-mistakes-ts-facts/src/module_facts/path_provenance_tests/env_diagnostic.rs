use super::{read, PathAnchor};

#[test]
fn captured_environment_details_do_not_restore_proof_provenance() {
    let facts = read("env-diagnostic.mjs");
    let captured = facts.origin("readdirSync(captured)");
    assert_eq!(captured.anchor, PathAnchor::Unknown);
    assert_eq!(captured.env, None);
    let capture = captured.captured_env.as_deref().unwrap();
    assert_eq!(capture.name, "ROOT");
    assert_eq!(
        capture.offset,
        facts.source.find("process.env.ROOT ??").unwrap() as u32
    );
    let hint = captured.env_diagnostic.unwrap();
    assert_eq!(hint.name, "ROOT");
    assert_eq!(hint.fallback.as_deref(), Some("./tmp/example"));
    assert_eq!(hint.line, 4);
    assert!(hint.captured);

    let immediate = facts.origin("readdirSync(process.env.ROOT");
    assert_eq!(immediate.env.as_deref(), Some("ROOT"));
    let hint = immediate.env_diagnostic.unwrap();
    assert_eq!(hint.fallback.as_deref(), Some("./tmp/or"));
    assert!(!hint.captured);

    let computed = facts.origin("readdirSync(computed)");
    assert_eq!(computed.env, None);
    assert!(computed.env_diagnostic.unwrap().fallback.is_none());
}

#[test]
fn clearing_locations_are_retained_as_diagnostics_only() {
    let facts = read("env-diagnostic.mjs");
    let writes = &facts.provenance().env_writes;
    assert_eq!(writes.len(), 2);
    assert!(writes
        .iter()
        .all(|write| write.name == "ROOT" && write.cleared));
    assert_eq!(
        writes.iter().map(|write| write.line).collect::<Vec<_>>(),
        [7, 8]
    );
    assert!(writes
        .iter()
        .all(|write| write.origin.anchor == PathAnchor::Unknown));
}

#[test]
fn captured_alternatives_do_not_claim_a_unique_environment_root() {
    let facts = read("env-diagnostic-ambiguous.mjs");
    for call in [
        "readdirSync(multiple)",
        "readdirSync(mixed)",
        "readdirSync(aliasOr)",
        "readdirSync(aliasCoalesce)",
        "readdirSync(aliasUnknown)",
    ] {
        let origin = facts.origin(call);
        assert_eq!(origin.anchor, PathAnchor::Unknown);
        assert_eq!(origin.env, None);
        assert!(origin.captured_env.is_none(), "{call}");
        assert!(origin.env_diagnostic.is_none(), "{call}");
        assert!(origin.captured_getter.is_none(), "{call}");
    }
}
