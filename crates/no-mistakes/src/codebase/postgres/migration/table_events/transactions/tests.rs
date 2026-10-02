use super::{Command, Marker};

fn fixture(file: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/table-event-forms")
            .join(file),
    )
    .unwrap()
}

fn marker(order: usize, command: Command) -> Marker {
    Marker {
        order: vec![order],
        command,
    }
}

fn retained_events(orders: &[Vec<usize>], markers: &[Marker]) -> Vec<bool> {
    let borrowed = orders.iter().map(Vec::as_slice).collect::<Vec<_>>();
    super::retained_events(&borrowed, markers)
}

#[test]
fn malformed_and_atomic_statements_are_not_transaction_markers() {
    let sql = fixture("invalid-transaction-markers.sql");
    let tokens = crate::codebase::postgres::parse::unicode::tokenize_raw_unicode(&sql);
    for (ordinal, statement) in tokens
        .split(|token| matches!(token.token, sqlparser::tokenizer::Token::SemiColon))
        .enumerate()
    {
        let code = statement
            .iter()
            .filter(|token| !matches!(token.token, sqlparser::tokenizer::Token::Whitespace(_)))
            .collect::<Vec<_>>();
        assert!(super::parse(&code, vec![ordinal]).is_none());
    }
}

#[test]
fn rollback_discards_only_events_inside_the_transaction() {
    let retained = retained_events(
        &[vec![0], vec![2], vec![4], vec![6]],
        &[
            marker(1, Command::Begin),
            marker(3, Command::Rollback),
            marker(5, Command::Begin),
            marker(7, Command::Commit),
        ],
    );
    assert_eq!(retained, [true, false, true, true]);
}

#[test]
fn savepoint_rollback_keeps_prior_and_later_committed_events() {
    let retained = retained_events(
        &[vec![2], vec![4], vec![6], vec![9]],
        &[
            marker(1, Command::Begin),
            marker(3, Command::Savepoint("sp".into())),
            marker(7, Command::RollbackTo("sp".into())),
            marker(10, Command::Commit),
        ],
    );
    assert_eq!(retained, [true, false, false, true]);
}

#[test]
fn an_open_transaction_is_retained_conservatively() {
    assert_eq!(
        retained_events(&[vec![2]], &[marker(1, Command::Begin)]),
        [true]
    );
}

#[test]
fn unknown_locations_and_savepoints_are_kept_conservatively() {
    assert_eq!(
        retained_events(
            &[vec![usize::MAX], vec![2], vec![4]],
            &[
                marker(1, Command::Begin),
                marker(3, Command::RollbackTo("unknown".into())),
                marker(5, Command::Release("unknown".into())),
            ],
        ),
        [true, true, true]
    );
}

#[test]
fn repeated_begin_does_not_create_a_nested_transaction() {
    assert_eq!(
        retained_events(
            &[vec![4]],
            &[
                marker(1, Command::Begin),
                marker(2, Command::Begin),
                marker(5, Command::Rollback),
            ],
        ),
        [false]
    );
}
