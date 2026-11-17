#[test]
fn splicing_fills_every_hole_and_renumbers_the_generated_binds() {
    let p = INTERNAL_PLACEHOLDER;
    let internal = format!(
        "SELECT 1 WHERE a = {p}1 AND {} AND b = {p}2",
        sentinel::encode(0)
    );
    let template = template_call(internal);
    let owner = Some(PathBuf::from("/helper.ts"));
    let hole = plain(
        format!("c = {p}1"),
        Marker {
            line: 3,
            path: owner.clone(),
            origin: 0,
        },
    );
    let mut holes = crate::fx::FxHashMap::default();
    holes.insert(0, vec![hole]);
    let call = splice(&template, &holes).expect("complete");
    assert_eq!(
        call.sql_text.as_deref(),
        Some(
            "SELECT 1 WHERE a = sql_placeholder_1 AND c = sql_placeholder_2 AND b = sql_placeholder_3"
        )
    );
    assert_eq!(call.kind, EmbeddedSqlKind::Composed);
    assert_eq!(call.recovered_placeholder_positions.len(), 3);
    assert!(call
        .sql_source_positions
        .iter()
        .any(|position| position.source_path == owner && position.source_line == 3));
    // Any gap leaves the call as the walk published it.
    assert!(splice(&template, &Default::default()).is_none());
    let mut no_text = template.clone();
    no_text.sql_text = None;
    assert!(splice(&no_text, &holes).is_none());
    let mut drifted = template.clone();
    drifted.recovered_placeholder_positions = vec![(1, 2)];
    assert!(splice(&drifted, &holes).is_none());
    // A template holding no hole at all has nothing to compose from.
    let empty = template_call(String::new());
    assert!(splice(&empty, &holes).is_none());
}

#[test]
fn extracted_calls_never_expose_a_hole_marker() {
    let facts = extract("pending.ts");
    let leaks = |text: &str| text.contains(MARK);
    assert!(facts
        .calls
        .iter()
        .all(|call| call.sql_text.as_deref().is_none_or(|text| !leaks(text))));
    assert!(facts
        .fragments
        .iter()
        .all(|fragment| fragment.sql_text.as_deref().is_none_or(|text| !leaks(text))));
    // Whatever still waits on a helper is, until it resolves, plainly `Dynamic`.
    assert!(facts
        .calls
        .iter()
        .zip(&facts.call_spans)
        .filter(|(_, span)| facts
            .pending_imported
            .iter()
            .any(|pending| pending.span == **span))
        .all(|(call, _)| call.kind == EmbeddedSqlKind::Dynamic));
}
