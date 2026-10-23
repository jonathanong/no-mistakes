use crate::codebase::check_facts::{collect_file_facts_from_program, CheckFactPlan};
use crate::codebase::postgres::EmbeddedSqlOptions;

#[test]
fn helper_demand_is_opt_in_profile_specific_and_reuses_the_prepared_walk() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("sql-literal-helpers/src/query.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let base = EmbeddedSqlOptions::configured("pg", &[])
        .with_scoped_executors(&[], &["PoolClient".into()]);
    for enabled in [false, true] {
        let profile = if enabled {
            base.clone()
                .with_sql_literal_helpers(&["pg-copy-streams#from".into()])
        } else {
            base.clone()
        };
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let plan = CheckFactPlan {
            query_annotation: true,
            query_annotation_options: Some(vec![profile.clone()]),
            embedded_sql_options: vec![profile.clone()],
            ..Default::default()
        };
        let file = crate::diagnostics::with_observer(Some(observer.clone()), || {
            crate::ast::with_program(&path, &source, |program, source| {
                collect_file_facts_from_program(
                    path.parent().unwrap(),
                    &path,
                    &plan,
                    None,
                    source,
                    program,
                    None,
                )
            })
            .unwrap()
        });
        let work = observer.snapshot().work;
        if enabled {
            assert!(work["postgres.annotation.literal-helper.candidates"] > 0);
            assert!(file.ts.module_bindings.is_some());
            let span = file.embedded_sql[0].1.call_spans[0];
            let heads = file.query_annotation_heads.as_ref().unwrap();
            assert_eq!(
                heads.helper(span, &profile.sql_literal_helpers),
                Some(super::HeadProof::Literal)
            );
            assert_eq!(heads.helper(span, &[]), None);
            assert_eq!(heads.helper(span, &["other#from".into()]), None);
        } else {
            assert!(!work.contains_key("postgres.annotation.literal-helper.candidates"));
            assert!(
                file.query_annotation_heads.is_none(),
                "disabled helpers add no heads walk"
            );
            assert!(
                file.ts.module_bindings.is_none(),
                "disabled helpers add no ownership index"
            );
        }
    }
}

#[test]
fn helper_profiles_are_canonical() {
    let profile = EmbeddedSqlOptions::default().with_sql_literal_helpers(&[
        "b#y".into(),
        "a#x".into(),
        "b#y".into(),
    ]);
    assert_eq!(profile.sql_literal_helpers, ["a#x", "b#y"]);
}
