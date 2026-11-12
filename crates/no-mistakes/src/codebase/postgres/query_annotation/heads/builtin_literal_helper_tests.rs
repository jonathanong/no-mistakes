use crate::codebase::check_facts::{
    tests::compatibility::collect_file_facts_from_program, CheckFactPlan,
};
use crate::codebase::postgres::EmbeddedSqlOptions;

#[test]
fn builtin_helpers_require_runtime_imports_and_selected_annotation_profiles() {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation");
    let profile = EmbeddedSqlOptions::configured("pg", &[])
        .with_scoped_executors(&[], &["PoolClient".into()]);
    for (fixture, annotation, selected, expected) in [
        ("copy-streams/src/query.mts", true, true, true),
        ("copy-streams/src/query.mts", false, true, false),
        ("copy-streams/src/query.mts", true, false, false),
        (
            "copy-streams-import-gates/namespace-only.mts",
            true,
            true,
            true,
        ),
        (
            "copy-streams-import-gates/to-alias-only.mts",
            true,
            true,
            true,
        ),
        ("copy-streams-import-gates/type-only.mts", true, true, false),
        (
            "copy-streams-import-gates/inline-type-only.mts",
            true,
            true,
            false,
        ),
        (
            "copy-streams-import-gates/default-only.mts",
            true,
            true,
            false,
        ),
        (
            "copy-streams-import-gates/side-effect-only.mts",
            true,
            true,
            false,
        ),
        (
            "copy-streams-import-gates/unrelated-export.mts",
            true,
            true,
            false,
        ),
        (
            "copy-streams-import-gates/comment-only.mts",
            true,
            true,
            false,
        ),
    ] {
        let path = root.join(fixture);
        let source = std::fs::read_to_string(&path).unwrap();
        let plan = CheckFactPlan {
            query_annotation: annotation,
            query_annotation_options: Some(vec![if selected {
                profile.clone()
            } else {
                EmbeddedSqlOptions::configured("other", &[])
            }]),
            embedded_sql_options: vec![profile.clone()],
            ..Default::default()
        };
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let file = crate::diagnostics::with_observer(Some(observer.clone()), || {
            crate::ast::with_program(&path, &source, |program, source| {
                collect_file_facts_from_program(&root, &path, &plan, None, source, program, None)
            })
            .unwrap()
        });
        assert_eq!(file.query_annotation_heads.is_some(), expected, "{fixture}");
        assert_eq!(file.ts.module_bindings.is_some(), expected, "{fixture}");
        if expected {
            let heads = file.query_annotation_heads.as_ref().unwrap();
            let span = file.embedded_sql[0].1.call_spans[0];
            assert_eq!(heads.helper(span, &[]), None);
            assert_eq!(
                heads.annotation_helper(span, &[]),
                Some(super::HeadProof::Literal)
            );
        }
        assert_eq!(
            observer
                .snapshot()
                .work
                .contains_key("postgres.annotation.head_syntax_visits"),
            expected,
            "{fixture}: annotation={annotation}, selected={selected}"
        );
    }
}
