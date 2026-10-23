use super::*;

#[test]
fn tagged_template_uses_placeholder_convention() {
    let facts = extract("tagged-template.ts");
    assert_eq!(facts.executor_bindings, ["query"]);
    let call = facts
        .calls
        .iter()
        .find(|call| call.callee == "query")
        .unwrap();
    assert_eq!(
        call.sql_text.as_deref(),
        Some("SELECT id FROM users WHERE id = sql_placeholder_1")
    );
}

#[test]
fn template_literal_interpolations_are_numbered() {
    let facts = extract("interpolation.ts");
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT * FROM items WHERE id = sql_placeholder_1 AND status = sql_placeholder_2")
    );
}

#[test]
fn string_literal_query_is_preserved() {
    let facts = extract("string-literal.ts");
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT count(*) FROM users")
    );
}

#[test]
fn aliased_executor_import_binds_local_name() {
    let facts = extract("aliased-import.ts");
    assert_eq!(facts.executor_bindings, ["q"]);
    assert_eq!(facts.calls[0].callee, "q");
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM accounts")
    );
}

#[test]
fn with_transaction_binds_query() {
    let facts = extract("with-transaction.ts");
    assert_eq!(facts.executor_bindings, ["query"]);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("INSERT INTO t (id) VALUES ($1)")
    );
}

#[test]
fn missing_specifier_has_no_executor_bindings() {
    let facts = extract("missing-specifier.ts");
    assert!(facts.executor_bindings.is_empty());
    assert!(facts.calls.iter().all(|call| call.callee != "query"
        || call.sql_text.is_none()
        || facts.executor_bindings.contains(&call.callee)));
    assert!(facts.calls.is_empty());
}

#[test]
fn identifier_bindings_and_member_query_calls() {
    let facts = extract("identifier-binding.ts");
    let by_sql = |needle: &str| {
        facts
            .calls
            .iter()
            .find(|call| call.sql_text.as_deref() == Some(needle))
            .unwrap()
    };
    assert_eq!(
        by_sql("SELECT name FROM users WHERE id = sql_placeholder_1").callee,
        "query"
    );
    assert_eq!(by_sql("SELECT 2").callee, "read");
    assert_eq!(facts.calls.len(), 3);
}

#[test]
fn with_transaction_options_and_arrow_call() {
    let facts = extract("with-transaction-options.ts");
    assert_eq!(facts.executor_bindings, ["query"]);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT sql_placeholder_1")
    );
}

#[test]
fn sql_text_and_executed_query_text_helpers() {
    let allocator = Allocator::default();
    let source = "const a = 'SELECT 1'; const b = `x${y}`; const c = n;";
    let parsed = crate::ast::parse(
        Path::new("helpers.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    let mut inits = Vec::new();
    for statement in &parsed.program.body {
        if let oxc_ast::ast::Statement::VariableDeclaration(declaration) = statement {
            inits.push(declaration.declarations[0].init.as_ref().unwrap());
        }
    }
    assert_eq!(sql_text(inits[0]).as_deref(), Some("SELECT 1"));
    assert_eq!(sql_text(inits[1]).as_deref(), Some("xsql_placeholder_1"));
    assert!(sql_text(inits[2]).is_none());
    let mut bindings = HashMap::new();
    bindings.insert("n".to_string(), "SELECT bound".to_string());
    assert_eq!(
        executed_query_text(inits[2], &bindings).as_deref(),
        Some("SELECT bound")
    );
    assert_eq!(
        executed_query_text(inits[0], &bindings).as_deref(),
        Some("SELECT 1")
    );
    assert!(executed_query_text(inits[2], &HashMap::new()).is_none());
}

#[test]
fn query_text_lookup_does_not_certify_calls_properties_or_non_string_values() {
    let path = fixture("query-text-unknown-expression.ts");
    let source = read_fixture("query-text-unknown-expression.ts");
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(&path, &allocator, &source, SourceType::ts());
    let mut bindings = HashMap::new();
    bindings.insert("buildSql".into(), "SELECT 1".into());
    bindings.insert("object".into(), "SELECT 2".into());
    for statement in &parsed.program.body {
        if let oxc_ast::ast::Statement::VariableDeclaration(declaration) = statement {
            let expression = declaration.declarations[0].init.as_ref().unwrap();
            assert!(executed_query_text(expression, &bindings).is_none());
        }
    }
}
