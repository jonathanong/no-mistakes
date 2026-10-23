use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::rules::postgres_require_query_annotation::{compile_options, Options};

#[test]
fn configured_helper_lookup_without_prepared_file_or_heads_cannot_prove_sql() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("sql-literal-helpers/src/query.mts");
    let opts = compile_options(&Options {
        import_specifier: "pg".into(),
        sql_literal_helpers: vec!["pg-copy-streams#from".into()],
        ..Default::default()
    })
    .unwrap();
    let mut facts = CheckFactMap::default();
    assert_eq!(proof(&facts, &path, &opts, (0, 0)), None);
    facts
        .ts
        .insert(path.clone(), CheckFileFacts::default().into());
    assert_eq!(proof(&facts, &path, &opts, (0, 0)), None);
}
