use super::*;
use crate::codebase::postgres::embedded::{bindings, scoped_bindings, tests};

#[test]
fn disabled_lock_profile_skips_helper_summary_construction() {
    let path = tests::fixture("transaction-lock-profile.ts");
    let source = tests::read_fixture("transaction-lock-profile.ts");
    for (enabled, expected_passes) in [(false, 0), (true, 1)] {
        let mut options = EmbeddedSqlOptions::configured("@example/db", &[])
            .with_scoped_executors(&["beginTransaction".into()], &[]);
        options.track_transaction_helpers = enabled;
        crate::ast::with_program(&path, &source, |program, _| {
            let bindings = bindings::executor_bindings(program, &options);
            let scoped = scoped_bindings::collect(program, &options);
            let collected = collect_calls(
                program,
                &source,
                &bindings,
                &scoped.executors,
                &scoped.spans,
                &options,
                (false, false),
            );
            assert_eq!(collected.lock_summary_passes, expected_passes);
            assert!(!collected.calls.is_empty());
        })
        .unwrap();
    }
}
