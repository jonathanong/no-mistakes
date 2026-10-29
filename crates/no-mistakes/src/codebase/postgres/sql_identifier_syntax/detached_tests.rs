use super::*;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};

fn fixture(name: &str, profiles: &[EmbeddedSqlOptions], visit: impl FnOnce(FileFacts, &str)) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-test-state-mutation/detached-executors/tests")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let ts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        visit(collect(program, &ts, &source, profiles), &source);
    })
    .unwrap();
}
fn line(source: &str, marker: &str) -> u32 {
    (source
        .lines()
        .position(|line| line.contains(marker))
        .unwrap()
        + 1) as u32
}
fn profile() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@example/db", &[])
}
#[test]
fn exact_detached_methods_are_normal_inputs_and_trusted_sinks() {
    fixture("exact.test.ts", &[profile()], |facts, source| {
        let calls = &facts.profiles[0].1;
        for marker in [
            "detached_capture",
            "detached_alias",
            "detached_bound');",
            "detached_bound ADD",
            "copied_member",
            "rebound_method",
        ] {
            let call = calls
                .iter()
                .find(|call| call.line == line(source, marker))
                .unwrap();
            assert!(
                matches!(&call.input, Input::Sql { text, .. } if text.contains("DROP")
                || text.contains("TRUNCATE") || text.contains("ALTER")),
                "{marker}"
            );
            assert!(facts.recovery.sinks[&profile()].contains(&call.span));
        }
        assert_eq!(calls.len(), 7);
    });
}
#[test]
fn incomplete_detached_bindings_retain_unknown_at_invocation() {
    for (name, marker) in [
        ("wrong-this.test.ts", "wrongThis('"),
        ("prebound.test.ts", "alias('SELECT"),
        ("mutable.test.ts", "second('"),
        ("uncertain.test.ts", "changedAlias('"),
        ("after-copy.test.ts", "second('"),
        ("escaped.test.ts", "alias('"),
        ("receiver-escaped.test.ts", "alias('"),
        ("receiver-history.test.ts", "escaped_receiver_history"),
        ("receiver-origins.test.ts", "first_receiver_history"),
        ("receiver-origins.test.ts", "second_receiver_history"),
        ("receiver-escaped-after.test.ts", "alias('"),
        ("mixed-roles.test.ts", "mixed_receiver"),
        ("mixed-roles.test.ts", "mixed_callable"),
        ("mixed-roles.test.ts", "reverse_callable"),
        ("mixed-roles.test.ts", "reverse_receiver"),
        ("assigned.test.ts", "run('"),
        ("assigned.test.ts", "initialized('"),
        ("assigned.test.ts", "copied('"),
        ("assigned.test.ts", "last('"),
        ("assigned.test.ts", "callback('"),
        ("assigned.test.ts", "changing('"),
        ("assigned.test.ts", "after_unrelated_reassignment"),
    ] {
        fixture(name, &[profile()], |facts, source| {
            let call = facts.profiles[0]
                .1
                .iter()
                .find(|call| call.line == line(source, marker))
                .unwrap();
            assert!(matches!(call.input, Input::Unknown), "{name}");
            assert!(
                !facts.recovery.sinks[&profile()].contains(&call.span),
                "{name}"
            );
        });
    }
}
#[test]
fn invalid_uninvoked_binds_preserve_existing_executor_escape_advice() {
    fixture("legacy-escape.test.ts", &[profile()], |facts, source| {
        for marker in ["const bad =", "const namespaceBad ="] {
            assert!(
                facts.profiles[0]
                    .1
                    .iter()
                    .any(|call| call.line == line(source, marker)
                        && matches!(call.input, Input::Unknown)),
                "missing {marker}"
            );
        }
    });
    fixture("escape-only.test.ts", &[profile()], |facts, source| {
        for marker in [
            "export { namedEscaped",
            "const invalidEscaped =",
            "export { invalidEscaped",
        ] {
            assert!(
                facts.profiles[0]
                    .1
                    .iter()
                    .any(|call| call.line == line(source, marker)
                        && matches!(call.input, Input::Unknown)),
                "missing {marker}"
            );
        }
    });
}

#[test]
fn shared_static_shapes_retain_exact_and_uncertain_query_methods() {
    fixture("shape-forms.test.ts", &[profile()], |facts, source| {
        for marker in ["bracket('", "template('", "second('", "bound('"] {
            let call = facts.profiles[0]
                .1
                .iter()
                .find(|call| call.line == line(source, marker))
                .unwrap();
            assert!(matches!(call.input, Input::Sql { .. }), "{marker}");
            assert!(facts.recovery.sinks[&profile()].contains(&call.span));
        }
    });
    fixture("uncertain-shapes.test.ts", &[profile()], |facts, source| {
        for marker in ["fallback('", "uncertain('"] {
            let call = facts.profiles[0]
                .1
                .iter()
                .find(|call| call.line == line(source, marker))
                .unwrap();
            assert!(matches!(call.input, Input::Unknown), "{marker}");
            assert!(!facts.recovery.sinks[&profile()].contains(&call.span));
        }
    });
}

#[test]
fn called_bodies_retain_detached_roots_for_helper_demand() {
    fixture("local-bodies.test.ts", &[profile()], |facts, _| {
        let calls = &facts.profiles[0].1;
        let exact = calls.iter().find(|call| call.line == 5).unwrap();
        assert!(matches!(exact.input, Input::Sql { .. }));
        assert!(facts.recovery.sinks[&profile()].contains(&exact.span));
        let uncertain = calls.iter().find(|call| call.line == 11).unwrap();
        assert!(matches!(uncertain.input, Input::Unknown));
        assert!(!facts.recovery.sinks[&profile()].contains(&uncertain.span));
    });
}

mod controls;
