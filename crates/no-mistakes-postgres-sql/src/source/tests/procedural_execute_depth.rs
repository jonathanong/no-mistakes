use super::procedural_occurrences::{block, kinds};

fn nested_execute(levels: usize) -> String {
    let mut sql = "SELECT 1".to_string();
    for level in 0..levels {
        sql = format!("EXECUTE $q{level}${sql}$q{level}$");
    }
    // `$body$` so a nested `$qN$$qM$` pair does not close the outer quote.
    format!("DO $body$ BEGIN {sql}; END $body$;")
}

#[test]
fn one_literal_execute_select_still_classifies() {
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'SELECT 1'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
    assert!(parsed.complete, "{:?}", parsed.diagnostics);
}

#[test]
fn nested_literal_execute_shares_the_outer_budget() {
    // A two-level nest still promotes the inner SELECT. Nested literal facts
    // stay incomplete for a separate reason, so completeness is not asserted.
    let (_, parsed) = block(&nested_execute(2));
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
}

#[test]
fn exhausting_the_shared_budget_fails_closed() {
    // More than the 64-level walker budget. A reset counter would still promote
    // the inner SELECT to Utility. Sharing the budget stops and fails closed.
    let (_, parsed) = block(&nested_execute(80));
    assert_eq!(kinds(&parsed.occurrences), ["DynamicExecute"]);
    assert!(!parsed.complete, "{:?}", kinds(&parsed.occurrences));
}
