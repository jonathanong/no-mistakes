use super::*;

#[test]
fn an_unresolved_specifier_and_empty_or_unknown_owners_drop_the_call() {
    let root = PathBuf::from("/pkg");
    let inside = PathBuf::from("/pkg/src/transaction.ts");
    let mut facts = facts(
        Vec::new(),
        PendingRelativeScope {
            candidates: vec![
                candidate("../missing", "openTransaction", true, false),
                candidate("../transaction", "TxExecutor", false, true),
            ],
            calls: vec![
                PendingRelativeCall {
                    seq: 0,
                    owners: vec![0],
                    call: call("unresolved"),
                },
                PendingRelativeCall {
                    seq: 1,
                    owners: Vec::new(),
                    call: call("no-owner"),
                },
                PendingRelativeCall {
                    seq: 2,
                    owners: vec![9],
                    call: call("out-of-range"),
                },
                PendingRelativeCall {
                    seq: 3,
                    owners: vec![1, 9],
                    call: call("kept-by-sibling"),
                },
            ],
            ..PendingRelativeScope::default()
        },
    );
    project_relative_scoped_facts(&mut facts, Some(&root), |specifier| {
        (specifier == "../transaction").then(|| inside.clone())
    });
    assert_eq!(sqls(&facts), ["kept-by-sibling"]);
    assert!(facts.matched_factory_names.is_empty());
    assert_eq!(facts.matched_type_names, ["TxExecutor"]);
}

#[test]
fn promoted_calls_merge_by_sequence_and_a_mismatch_appends() {
    let root = PathBuf::from("/pkg");
    let inside = Some(PathBuf::from("/pkg/a.ts"));
    let mut ordered = facts(
        vec![call("second"), call("fourth")],
        PendingRelativeScope {
            candidates: vec![candidate("./a", "openTransaction", true, false)],
            calls: vec![
                PendingRelativeCall {
                    seq: 0,
                    owners: vec![0],
                    call: call("first"),
                },
                PendingRelativeCall {
                    seq: 2,
                    owners: vec![0],
                    call: call("third"),
                },
            ],
            confirmed_order: vec![1, 3],
            ..PendingRelativeScope::default()
        },
    );
    project(&mut ordered, Some(&root), inside.clone());
    assert_eq!(sqls(&ordered), ["first", "second", "third", "fourth"]);

    let mut tied = facts(
        vec![call("confirmed")],
        PendingRelativeScope {
            candidates: vec![candidate("./a", "openTransaction", false, false)],
            calls: vec![PendingRelativeCall {
                seq: 0,
                owners: vec![0],
                call: call("same-seq"),
            }],
            confirmed_order: vec![0],
            ..PendingRelativeScope::default()
        },
    );
    project(&mut tied, Some(&root), inside.clone());
    assert_eq!(sqls(&tied), ["confirmed", "same-seq"]);
    assert!(tied.matched_factory_names.is_empty());

    let mut mismatched = facts(
        vec![call("confirmed")],
        PendingRelativeScope {
            candidates: vec![candidate("./a", "openTransaction", true, false)],
            calls: vec![PendingRelativeCall {
                seq: 0,
                owners: vec![0],
                call: call("appended"),
            }],
            confirmed_order: Vec::new(),
            ..PendingRelativeScope::default()
        },
    );
    project(&mut mismatched, Some(&root), inside);
    assert_eq!(sqls(&mismatched), ["confirmed", "appended"]);
}

#[test]
fn nothing_promoted_leaves_confirmed_calls_in_place() {
    let mut facts = facts(
        vec![call("confirmed")],
        PendingRelativeScope {
            candidates: vec![candidate("../transaction", "openTransaction", true, false)],
            calls: Vec::new(),
            confirmed_order: vec![0],
            ..PendingRelativeScope::default()
        },
    );
    project(
        &mut facts,
        Some(Path::new("/pkg")),
        Some(PathBuf::from("/other/transaction.ts")),
    );
    assert_eq!(sqls(&facts), ["confirmed"]);
    assert!(facts.matched_factory_names.is_empty());
}

#[test]
fn empty_candidates_clear_pending_without_reading_the_resolver() {
    let mut facts = facts(
        vec![call("confirmed")],
        PendingRelativeScope {
            spans: vec![PendingRelativeSpan {
                owners: vec![0],
                name: "tx".to_string(),
                start: 0,
                end: 4,
            }],
            ..PendingRelativeScope::default()
        },
    );
    project_relative_scoped_facts(&mut facts, None, |_| panic!("resolver"));
    assert_eq!(sqls(&facts), ["confirmed"]);
    assert_eq!(facts.pending_relative, PendingRelativeScope::default());
}
