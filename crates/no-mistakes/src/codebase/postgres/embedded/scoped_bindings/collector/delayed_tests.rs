use super::ScopeCollector;
use crate::codebase::postgres::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use oxc_ast_visit::Visit;

#[test]
fn absent_keyword_keeps_parameter_proof_without_delayed_records() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("delayed-try-receivers/no-keyword.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    assert!(!source.contains("let"));
    let options = EmbeddedSqlOptions::configured("pg", &[])
        .with_scoped_executors(&[], &["PoolClient".into()]);
    crate::ast::with_program(&path, &source, |program, _| {
        let imports = super::super::candidates::classify(program, &options);
        let mut visitor =
            ScopeCollector::from_imports(imports, program.source_text.contains("let"));
        visitor.visit_program(program);
        assert_eq!(visitor.delayed, super::delayed::Bindings::default());
    })
    .unwrap();
    assert_eq!(
        extract_embedded_sql_from_source(&path, &source, &options)
            .calls
            .len(),
        1
    );
}
