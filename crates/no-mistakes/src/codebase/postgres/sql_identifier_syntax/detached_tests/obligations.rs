use super::*;

fn obligations(name: &str, visit: impl FnOnce(FileFacts, &str)) {
    let profile = EmbeddedSqlOptions::configured(
        "@example/db",
        &[
            "query".into(),
            "read".into(),
            "write".into(),
            "execute".into(),
        ],
    );
    fixture(
        &format!("../../detached-obligations/tests/{name}"),
        &[profile],
        visit,
    );
}
#[test]
fn non_direct_executor_references_have_independent_unknown_obligations() {
    obligations("references.test.ts", |facts, source| {
        let calls = &facts.profiles[0].1;
        for marker in [
            "pool.query.bind",
            "q.call",
            "q.apply",
            "Reflect.apply",
            "consume(pool.query)",
            "consume(q)",
            "export { q }",
            "function f(",
            "function g(",
            "query(sql)",
            "run(text)",
            "const alias",
            "consume(alias)",
            "const unused",
            "const exact",
            "function unusedDefault",
        ] {
            assert!(
                calls.iter().any(|call| call.line == line(source, marker)
                    && matches!(call.input, Input::Unknown)),
                "{marker}"
            );
        }
    });
}
#[test]
fn reference_obligations_follow_each_configured_executor_name() {
    obligations("names.test.ts", |facts, source| {
        for marker in [
            "pool.read.call",
            "put.apply",
            "consume(pool.execute)",
            "const run",
            "execute(sql)",
        ] {
            assert!(
                facts.profiles[0]
                    .1
                    .iter()
                    .any(|call| call.line == line(source, marker)
                        && matches!(call.input, Input::Unknown)),
                "{marker}"
            );
        }
    });
}
#[test]
fn unrelated_and_erased_references_never_become_sql_obligations() {
    obligations("controls.test.ts", |facts, _| {
        assert!(facts.profiles[0].1.is_empty())
    });
}

#[test]
fn exact_destructuring_retains_known_sql_and_erased_uses_stay_clear() {
    obligations("known.test.ts", |facts, _| {
        assert_eq!(facts.profiles[0].1.len(), 4);
        assert!(facts.profiles[0]
            .1
            .iter()
            .all(|call| matches!(call.input, Input::Sql { .. })));
    });
    obligations("type-uses.test.ts", |facts, _| {
        assert!(facts.profiles[0].1.is_empty())
    });
}

#[test]
fn wrappers_preserve_direct_callees_and_one_member_obligation() {
    obligations("direct-wrappers.test.ts", |facts, _| {
        let calls = &facts.profiles[0].1;
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].line, 2);
        assert_eq!(calls[1].line, 4);
        // Namespace captures retain LOW's existing unsupported classification.
        assert!(calls
            .iter()
            .all(|call| matches!(call.input, Input::Unknown)));
    });
    obligations("wrapped-reference.test.ts", |facts, _| {
        assert_eq!(facts.profiles[0].1.len(), 1);
        assert!(matches!(facts.profiles[0].1[0].input, Input::Unknown));
    });
}

#[test]
fn exported_aliases_retain_obligations_before_their_capture_declaration() {
    obligations("forward-export.test.ts", |facts, _| {
        assert!(facts.profiles[0]
            .1
            .iter()
            .any(|call| call.line == 2 && matches!(call.input, Input::Unknown)));
    });
}

#[test]
fn detached_executor_names_work_without_query_and_do_not_cross_profiles() {
    let read = EmbeddedSqlOptions::configured("@example/db", &["read".into()]);
    let execute = EmbeddedSqlOptions::configured("@example/db", &["execute".into()]);
    let name = "../../detached-obligations/tests/known.test.ts";
    fixture(name, std::slice::from_ref(&execute), |facts, _| {
        assert_eq!(facts.profiles[0].1.len(), 1);
        assert_eq!(facts.profiles[0].1[0].line, 6);
        assert!(matches!(facts.profiles[0].1[0].input, Input::Sql { .. }));
    });
    fixture(name, &[read, execute], |facts, _| {
        for (index, expected) in [4, 6].into_iter().enumerate() {
            assert_eq!(facts.profiles[index].1.len(), 1);
            assert_eq!(facts.profiles[index].1[0].line, expected);
            assert!(matches!(
                facts.profiles[index].1[0].input,
                Input::Sql { .. }
            ));
        }
    });
}

#[test]
fn default_binding_transfers_keep_receiver_and_executor_potential() {
    obligations("parameter-alias.test.ts", |facts, _| {
        for line in [2, 3, 6, 7] {
            assert!(
                facts.profiles[0]
                    .1
                    .iter()
                    .any(|call| call.line == line && matches!(call.input, Input::Unknown)),
                "{line}"
            );
        }
    });
}
