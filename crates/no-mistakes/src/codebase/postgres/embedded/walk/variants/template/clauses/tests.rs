use super::*;
use crate::codebase::postgres::embedded::tests::{fixture, read_fixture};
use crate::codebase::postgres::embedded::{extract_embedded_sql_from_source, EmbeddedSqlOptions};

#[test]
fn unknown_callees_skip_argument_classification() {
    let path = fixture("identifier-clause-lazy-args.ts");
    let source = read_fixture("identifier-clause-lazy-args.ts");
    crate::ast::with_program(&path, &source, |program, source| {
        let functions = LocalFunctions::collect(program, source, &[]);
        let classifications = std::cell::Cell::new(0);
        assert_eq!(
            known_helper_args(&functions, "knownTable", || {
                classifications.set(classifications.get() + 1);
                Some(Vec::<()>::new())
            }),
            Some(vec![])
        );
        assert!(known_helper_args(&functions, "missing", || -> Option<()> {
            panic!("unknown helpers must not classify arguments")
        })
        .is_none());
        assert_eq!(classifications.get(), 1);
    })
    .expect("saved fixture parses");

    let calls = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    )
    .calls;
    assert_eq!(calls.len(), 2, "{calls:#?}");
    assert_eq!(calls[0].variants.len(), 1, "known helpers still recover");
    assert!(calls[0].variants[0].sql_text.contains("FROM orders"));
    assert!(calls[1].is_unanalyzable(), "{:#?}", calls[1]);
}
