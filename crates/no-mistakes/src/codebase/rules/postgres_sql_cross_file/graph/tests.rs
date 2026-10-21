#[test]
fn graph_plan_requires_a_configured_sql_cross_file_rule() {
    assert!(super::graph_plan(&crate::config::v2::NoMistakesConfig::default()).is_none());
}
