use super::*;

#[test]
fn unresolved_imported_calls_keep_the_pre_recovery_public_facts() {
    // The pending template retains the suffix; unresolved public facts stop at the fragment.
    let facts = extract("pending.ts");
    let mut snapshot = String::new();
    for (index, call) in facts.calls.iter().enumerate() {
        snapshot.push_str(&format!(
            "{index}\t{:?}\t{:?}\t{:?}\t{}\n",
            call.kind,
            call.sql_text,
            call.recovered_placeholder_positions,
            call.variants.len()
        ));
    }
    let baseline = fixture("pending-unresolved.txt");
    let saved = std::fs::read_to_string(&baseline).unwrap_or_default();
    assert_eq!(
        snapshot, saved,
        "public facts for unresolved imported helpers drifted from {baseline:?}"
    );
}

#[test]
fn the_value_of_an_arbitrary_imported_call_is_not_a_sql_builder() {
    let facts = extract("pending.ts");
    let source = std::fs::read_to_string(fixture("pending.ts")).expect("fixture");
    let line = source
        .lines()
        .position(|line| line.contains("element.append"))
        .expect("append") as u32
        + 1;
    // Were `element` treated as a builder, this append would be a recorded fragment.
    assert!(facts.fragments.iter().all(|fragment| fragment.line != line));
}

#[test]
fn only_value_imports_can_supply_a_hole() {
    let facts = extract("imports.ts");
    let locals = (0..facts.calls.len())
        .map(|index| pending_for(&facts, index).map(|pending| pending.sites[0].1.local.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        locals,
        [
            None,
            None,
            Some("value"),
            Some("alias"),
            None,
            Some("defaultHelper"),
            None
        ]
    );
}

#[test]
fn an_invalid_template_escape_is_not_spliced() {
    let facts = extract("uncooked-template.ts");
    assert!(facts.pending_imported.is_empty());
    assert!(facts.calls.iter().all(|call| {
        call.sql_text
            .as_deref()
            .is_none_or(|sql| !sql.contains("ORDER BY"))
    }));
}

#[test]
fn imported_helper_arguments_accept_shallow_values_and_reject_untracked_shapes() {
    let facts = extract("argument-boundaries.ts");
    assert_eq!(facts.calls.len(), 11);
    let accepted = pending_for(&facts, 0).expect("shallow values have a helper hole");
    assert_eq!(accepted.sites.len(), 1);
    assert_eq!(
        accepted.sites[0].1.args,
        vec![
            SiteArg::Undefined,
            SiteArg::Scalar,
            SiteArg::Array(vec![
                SiteArg::Scalar,
                SiteArg::Bool(true),
                SiteArg::Null,
                SiteArg::Undefined,
            ]),
            SiteArg::Object(vec![("id".into(), SiteArg::Scalar)]),
        ]
    );
    // Builders in containers, nested containers, spread, computed/numeric keys,
    // methods and accessors cannot be tracked as literal helper arguments.
    for index in 1..facts.calls.len() {
        assert!(pending_for(&facts, index).is_none(), "call {index}");
        assert_eq!(call(&facts, index).kind, EmbeddedSqlKind::Dynamic);
        assert_eq!(
            call(&facts, index).sql_text.as_deref(),
            Some("SELECT 1 WHERE "),
            "call {index}"
        );
        assert!(call(&facts, index).variants.is_empty(), "call {index}");
    }
    assert_eq!(facts.pending_imported.len(), 1);
}
