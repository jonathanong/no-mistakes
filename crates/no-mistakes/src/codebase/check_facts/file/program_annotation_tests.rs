use super::*;
use crate::codebase::postgres::EmbeddedSqlRuleScan;

#[test]
fn unspecified_direct_annotation_demand_keeps_all_profile_root_compatibility() {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-excluded-roots");
    let path = root.join("src/root.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let mut plan = CheckFactPlan {
        query_annotation: true,
        embedded_sql: true,
        embedded_sql_options: vec![options.clone()],
        embedded_sql_rule_ids: [(
            options.clone(),
            vec![EmbeddedSqlRuleScan::new("postgres-no-offset", &[], &[]).unwrap()],
        )]
        .into(),
        ..Default::default()
    };
    crate::ast::with_program(&path, &source, |program, _| {
        let embedded = [(
            options.clone(),
            crate::codebase::postgres::extract_embedded_sql_from_program(
                &path, program, &source, &options,
            ),
        )];
        let direct = collect(&root, &path, &source, program, &plan, &embedded);
        assert_eq!(direct[0].1.in_scope, None);
        plan.query_annotation_options = Some(vec![options.clone()]);
        let configured = collect(&root, &path, &source, program, &plan, &embedded);
        assert_eq!(configured[0].1.in_scope, Some(false));
    })
    .unwrap();
}
