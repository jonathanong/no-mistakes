#[test]
fn cov_p2_async_setup_preserves_heap_start_and_statement_budget() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("cov-p2-import-gate/async-bounds.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let mut checked = 0;
        for statement in &program.body {
            let oxc_ast::ast::Statement::FunctionDeclaration(function) = statement else {
                continue;
            };
            let name = function.id.as_ref().unwrap().name.as_str();
            assert_eq!(
                super::function(function).is_some(),
                name.starts_with("yes"),
                "{name}"
            );
            checked += 1;
        }
        assert_eq!(checked, 4);
    })
    .unwrap();
}
