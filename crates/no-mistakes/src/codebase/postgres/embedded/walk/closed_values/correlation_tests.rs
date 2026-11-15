fn texts(index: usize) -> Vec<String> {
    super::fixture::texts("query.ts", index)
}

#[test]
fn record_and_helper_keep_the_argument_choice() {
    assert_eq!(
        texts(0),
        [
            "DELETE FROM users WHERE user_id = $1",
            "DELETE FROM posts WHERE post_id = $1"
        ]
    );
}

#[test]
fn two_helpers_keep_the_same_argument_choice() {
    assert_eq!(
        texts(1),
        [
            "DELETE FROM users WHERE user_id = $1",
            "DELETE FROM posts WHERE post_id = $1"
        ]
    );
}

fn stays_unresolved(needle: &str) {
    let call = super::fixture::call_containing("query.ts", needle);
    assert!(call.variants.is_empty(), "{needle} {call:#?}");
    assert!(call.is_unanalyzable(), "{needle} {call:#?}");
}

#[test]
fn transformed_argument_stays_unresolved() {
    stays_unresolved("kind === \"user\"");
}

#[test]
fn reassigned_argument_stays_unresolved() {
    stays_unresolved("column(k)}");
}

#[test]
fn unmatched_literal_does_not_clear_the_statement() {
    stays_unresolved("column(\"missing\")");
}

#[test]
fn trimmed_argument_stays_unresolved() {
    stays_unresolved("kind.trim()");
}

#[test]
fn template_argument_stays_unresolved() {
    stays_unresolved("`${kind}`");
}

#[test]
fn empty_guard_does_not_clear_the_statement() {
    stays_unresolved("admitNone(kind)");
}

#[test]
fn type_assertion_keeps_the_argument_choice() {
    let mut values = super::fixture::texts_containing("query.ts", "kind as Key");
    values.sort();
    assert_eq!(
        values,
        [
            "DELETE FROM posts WHERE post_id = $1",
            "DELETE FROM users WHERE user_id = $1"
        ]
    );
}

#[test]
fn distinct_parameters_keep_the_cross_pairs() {
    let mut values = texts(2);
    values.sort();
    assert_eq!(
        values,
        [
            "DELETE FROM posts WHERE post_id = $1",
            "DELETE FROM posts WHERE user_id = $1",
            "DELETE FROM users WHERE post_id = $1",
            "DELETE FROM users WHERE user_id = $1"
        ]
    );
}
