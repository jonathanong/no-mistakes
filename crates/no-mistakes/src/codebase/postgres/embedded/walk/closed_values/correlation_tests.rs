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

fn stays_unresolved(index: usize) {
    let call = super::fixture::call("query.ts", index);
    assert!(call.variants.is_empty(), "{call:#?}");
    assert!(call.is_unanalyzable(), "{call:#?}");
}

#[test]
fn transformed_argument_stays_unresolved() {
    stays_unresolved(3);
}

#[test]
fn reassigned_argument_stays_unresolved() {
    stays_unresolved(4);
}

#[test]
fn unmatched_literal_does_not_clear_the_statement() {
    stays_unresolved(5);
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
