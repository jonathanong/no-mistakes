use super::*;
use crate::codebase::postgres::embedded::EmbeddedSqlKind;

#[test]
fn unconfirmed_relative_consumers_invalidate_builders_but_preserve_copied_sql() {
    let pending = extract(
        "variants-relative-consumer-escape.ts",
        &scoped("@example/db"),
    );
    let source = std::fs::read_to_string(embedded_fixture("variants-relative-consumer-escape.ts"))
        .expect("fixture");
    let lines = source.lines().collect::<Vec<_>>();
    assert_eq!(pending.calls.len(), 10);
    assert_eq!(pending.pending_relative.calls.len(), 4);
    for call in pending
        .calls
        .iter()
        .chain(pending.pending_relative.calls.iter().map(|call| &call.call))
    {
        let line = lines[call.line as usize - 1];
        if line.ends_with("finite") {
            assert_eq!(call.variants.len(), 2, "{call:?}");
        } else {
            assert!(line.ends_with("opaque"), "{line}");
            assert_eq!(call.kind, EmbeddedSqlKind::Dynamic, "{call:?}");
            assert!(call.variants.is_empty(), "{call:?}");
        }
    }
    let without_candidates = extract(
        "variants-relative-consumer-escape.ts",
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    // A provisional owner grants no additional read-only guarantee over an
    // ordinary unknown consumer. Compare every public field, including variants.
    assert_eq!(pending.calls, without_candidates.calls);
    assert_eq!(pending.call_spans, without_candidates.call_spans);

    let root = PathBuf::from("/virtual/db");
    let mut rejected = pending.clone();
    project(
        &mut rejected,
        Some(&root),
        Some(PathBuf::from("/virtual/other/transaction.ts")),
    );
    assert_eq!(rejected.calls, pending.calls);
    assert_eq!(rejected.call_spans, pending.call_spans);
    assert!(rejected.matched_factory_names.is_empty());
    assert!(rejected.matched_type_names.is_empty());

    let mut accepted = pending.clone();
    project(
        &mut accepted,
        Some(&root),
        Some(root.join("transaction.ts")),
    );
    assert_eq!(accepted.calls.len(), 14);
    assert_eq!(accepted.matched_factory_names, ["openTransaction"]);
    assert_eq!(accepted.matched_type_names, ["TxExecutor"]);
    let mut captured = pending.calls.clone();
    captured.extend(
        pending
            .pending_relative
            .calls
            .iter()
            .map(|call| call.call.clone()),
    );
    captured.sort_by_key(|call| call.line);
    assert_eq!(accepted.calls, captured);
    assert!(accepted
        .call_spans
        .windows(2)
        .all(|pair| pair[0].0 < pair[1].0));
    assert_eq!(accepted.pending_relative, PendingRelativeScope::default());

    let mut unknown = pending.clone();
    project(&mut unknown, None, None);
    assert_eq!(unknown.calls, pending.calls);
    assert_eq!(unknown.call_spans, pending.call_spans);
}

#[test]
fn pending_relative_calls_capture_argument_mutations_before_the_consumer_escape() {
    let pending = extract(
        "variants-relative-consumer-arguments.ts",
        &scoped("@example/db"),
    );
    assert_eq!(pending.calls.len(), 2);
    assert!(pending.calls.iter().all(EmbeddedSqlCall::is_unanalyzable));
    assert_eq!(pending.pending_relative.calls.len(), 2);
    for (index, call) in pending.pending_relative.calls.iter().enumerate() {
        let mut sql = call
            .call
            .variants
            .iter()
            .map(|variant| variant.sql_text.as_str())
            .collect::<Vec<_>>();
        sql.sort_unstable();
        let expected = if index == 0 {
            [
                "SELECT id FROM factory_users OFFSET 1",
                "SELECT id FROM factory_users WHERE id = 1 OFFSET 1",
            ]
        } else {
            [
                "SELECT id FROM typed_users OFFSET 2",
                "SELECT id FROM typed_users WHERE id = 2 OFFSET 2",
            ]
        };
        assert_eq!(sql, expected, "{call:#?}");
    }
    let root = PathBuf::from("/virtual/db");
    let mut accepted = pending.clone();
    project(
        &mut accepted,
        Some(&root),
        Some(root.join("transaction.ts")),
    );
    assert_eq!(accepted.calls.len(), 4);
    for index in 0..2 {
        assert_eq!(
            accepted.calls[index * 2],
            pending.pending_relative.calls[index].call
        );
        assert_eq!(accepted.calls[index * 2 + 1], pending.calls[index]);
    }
    assert!(accepted
        .call_spans
        .windows(2)
        .all(|pair| pair[0].0 < pair[1].0));
    let mut rejected = pending.clone();
    project(
        &mut rejected,
        Some(&root),
        Some(PathBuf::from("/virtual/other/transaction.ts")),
    );
    assert_eq!(rejected.calls, pending.calls);
    assert_eq!(rejected.call_spans, pending.call_spans);
}
