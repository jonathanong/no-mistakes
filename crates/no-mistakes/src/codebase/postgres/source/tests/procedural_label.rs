use super::procedural_occurrences::{block, kinds};

fn fails_closed(sql: &str) {
    let (_, parsed) = block(sql);
    assert!(!parsed.complete, "{sql}");
    assert!(
        parsed.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("Unsupported procedural occurrence")),
        "{:?}",
        parsed.diagnostics
    );
    assert!(
        parsed.diagnostics.iter().all(|diagnostic| {
            !diagnostic
                .message
                .contains("Procedural control flow is unsupported")
        }),
        "{:?}",
        parsed.diagnostics
    );
    assert!(
        kinds(&parsed.occurrences)
            .iter()
            .any(|kind| kind.contains("Unknown")),
        "{:?}",
        kinds(&parsed.occurrences)
    );
}

#[test]
fn mismatched_block_label_is_not_complete() {
    fails_closed("DO $$ <<foo>> BEGIN RAISE NOTICE 'x'; END bar; $$;");
}

#[test]
fn empty_label_before_raise_inside_loop_is_not_complete() {
    fails_closed("DO $$ BEGIN LOOP <<>> RAISE NOTICE 'x'; END LOOP; END $$;");
}

#[test]
fn empty_block_label_is_not_complete() {
    fails_closed("DO $$ <<>> BEGIN RAISE NOTICE 'x'; END; $$;");
    fails_closed("DO $$ << >> BEGIN RAISE NOTICE 'x'; END; $$;");
}

#[test]
fn label_that_is_not_one_identifier_is_not_complete() {
    fails_closed("DO $$ <<foo bar>> BEGIN RAISE NOTICE 'x'; END; $$;");
    fails_closed("DO $$ <<1>> BEGIN RAISE NOTICE 'x'; END; $$;");
    let (_, unclosed) = block("DO $$ <<foo BEGIN RAISE NOTICE 'x'; END; $$;");
    assert!(!unclosed.complete, "{:?}", unclosed.diagnostics);
}

#[test]
fn matching_block_label_stays_complete_control_flow() {
    let (_, parsed) = block("DO $$ <<foo>> BEGIN RAISE NOTICE 'x'; END foo; $$;");
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow"]);
}

#[test]
fn omitted_closing_label_stays_complete() {
    let (_, parsed) = block("DO $$ <<foo>> BEGIN RAISE NOTICE 'x'; END; $$;");
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow"]);

    let (_, spaced) = block("DO $$ << foo >> BEGIN RAISE NOTICE 'x'; END foo; $$;");
    assert!(spaced.complete, "{:?}", spaced.diagnostics);
}

#[test]
fn unquoted_labels_match_ascii_case_insensitively() {
    let (_, parsed) = block("DO $$ <<Foo>> BEGIN RAISE NOTICE 'x'; END foo; $$;");
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow"]);
}

#[test]
fn quoted_labels_match_only_the_same_quoted_word() {
    let (_, matched) = block(r#"DO $$ <<"Retry">> BEGIN RAISE NOTICE 'x'; END "Retry"; $$;"#);
    assert!(matched.complete, "{:?}", matched.diagnostics);
    assert_eq!(kinds(&matched.occurrences), ["ControlFlow"]);

    fails_closed(r#"DO $$ <<"Retry">> BEGIN RAISE NOTICE 'x'; END retry; $$;"#);
    fails_closed(r#"DO $$ <<"Retry">> BEGIN RAISE NOTICE 'x'; END "retry"; $$;"#);
    fails_closed(r#"DO $$ <<foo>> BEGIN RAISE NOTICE 'x'; END "foo"; $$;"#);
}

#[test]
fn matching_loop_label_on_null_does_not_add_a_failure() {
    // NULL stays the only unknown child. A matching label is not its own occurrence.
    let (sql, parsed) = block("DO $$ BEGIN <<retry>> LOOP NULL; END LOOP retry; END $$;");
    assert_eq!(parsed.occurrences.len(), 1);
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Unknown\"]"]);
    let span = &parsed.occurrences[0].span;
    assert!(sql[span.start.offset..span.end.offset].contains("END LOOP retry"));
    assert!(!parsed.complete);
    assert!(parsed.statements.is_empty());
    assert_eq!(
        parsed
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic
                .message
                .contains("Unsupported procedural occurrence"))
            .count(),
        1
    );
}

#[test]
fn quoted_loop_label_stays_one_control_flow_occurrence() {
    let (_, parsed) =
        block(r#"DO $$ BEGIN <<"Retry">> LOOP RAISE NOTICE 'x'; END LOOP "Retry"; END $$;"#);
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"ControlFlow\"]"]);

    fails_closed(r#"DO $$ BEGIN <<"Retry">> LOOP RAISE NOTICE 'x'; END LOOP retry; END $$;"#);
}

#[test]
fn closing_label_without_an_opening_label_stays_complete() {
    let (_, loop_label) = block("DO $$ BEGIN LOOP RAISE NOTICE 'x'; END LOOP retry; END $$;");
    assert!(loop_label.complete, "{:?}", loop_label.diagnostics);
    assert_eq!(
        kinds(&loop_label.occurrences),
        ["ControlFlow[\"ControlFlow\"]"]
    );

    let (_, block_label) = block("DO $$ BEGIN RAISE NOTICE 'x'; END retry; $$;");
    assert!(block_label.complete, "{:?}", block_label.diagnostics);
    assert_eq!(kinds(&block_label.occurrences), ["ControlFlow"]);
}

#[test]
fn statement_label_does_not_steal_the_block_label() {
    let (_, kept) = block("DO $$ <<foo>> BEGIN <<bar>> RAISE NOTICE 'x'; END foo; $$;");
    assert!(kept.complete, "{:?}", kept.diagnostics);
    assert_eq!(kinds(&kept.occurrences), ["ControlFlow"]);

    fails_closed("DO $$ <<foo>> BEGIN <<bar>> RAISE NOTICE 'x'; END bar; $$;");
    fails_closed("DO $$ <<foo>> BEGIN <<>> RAISE NOTICE 'x'; END foo; $$;");
}

#[test]
fn loop_and_nested_block_labels_use_their_own_closers() {
    let (_, matched) = block("DO $$ BEGIN <<foo>> LOOP RAISE NOTICE 'x'; END LOOP foo; END $$;");
    assert!(matched.complete, "{:?}", matched.diagnostics);
    assert_eq!(
        kinds(&matched.occurrences),
        ["ControlFlow[\"ControlFlow\"]"]
    );

    let (_, omitted) = block("DO $$ BEGIN <<foo>> LOOP RAISE NOTICE 'x'; END LOOP; END $$;");
    assert!(omitted.complete, "{:?}", omitted.diagnostics);

    fails_closed("DO $$ BEGIN <<foo>> LOOP RAISE NOTICE 'x'; END LOOP bar; END $$;");
    fails_closed("DO $$ BEGIN <<Foo>> LOOP <<bar>> RAISE NOTICE 'x'; END LOOP bar; END $$;");

    let (_, nested) = block("DO $$ BEGIN <<foo>> BEGIN RAISE NOTICE 'x'; END foo; END $$;");
    assert!(nested.complete, "{:?}", nested.diagnostics);
    assert_eq!(kinds(&nested.occurrences), ["ControlFlow[\"ControlFlow\"]"]);
    fails_closed("DO $$ BEGIN <<foo>> BEGIN RAISE NOTICE 'x'; END bar; END $$;");
}

#[test]
fn label_prefix_on_literal_execute_is_not_dml() {
    // Skipping `<<>>` or `<<mark>>` must not classify the command that follows.
    let (_, insert) = block("DO $$ BEGIN EXECUTE '<<>> INSERT INTO t VALUES (1)'; END $$;");
    assert_eq!(kinds(&insert.occurrences), ["Unknown"]);
    assert!(!insert.complete, "{:?}", insert.diagnostics);

    let (_, create) = block("DO $$ BEGIN EXECUTE '<<>> CREATE TABLE a(id int)'; END $$;");
    assert_eq!(kinds(&create.occurrences), ["Unknown"]);
    assert!(!create.complete, "{:?}", create.diagnostics);

    let (_, marked) = block("DO $$ BEGIN EXECUTE '<<mark>> INSERT INTO t VALUES (1)'; END $$;");
    assert_eq!(kinds(&marked.occurrences), ["Unknown"]);
    assert!(!marked.complete, "{:?}", marked.diagnostics);
}
