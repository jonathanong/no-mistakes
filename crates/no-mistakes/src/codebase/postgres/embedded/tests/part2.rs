use super::*;

#[test]
fn custom_specifier_and_executor_names() {
    let source = "import { run as r, withTransaction } from '@app/db'\nrun('no')\nr('yes')\n";
    let options = EmbeddedSqlOptions {
        import_specifier: "@app/db".to_string(),
        executor_names: vec!["run".to_string()],
        ..Default::default()
    };
    let facts = extract_embedded_sql_from_source(Path::new("custom.ts"), source, &options);
    assert!(facts.executor_bindings.iter().any(|name| name == "r"));
    assert!(facts.executor_bindings.iter().any(|name| name == "query"));
    assert_eq!(
        facts
            .calls
            .iter()
            .map(|c| c.callee.as_str())
            .collect::<Vec<_>>(),
        ["r"]
    );
}

#[test]
fn default_and_namespace_imports_are_ignored() {
    let source = "import db, * as all from '@example/db'\ndb('x')\nall.query('y')\n";
    let facts = extract_embedded_sql_from_source(
        Path::new("ns.ts"),
        source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert!(facts.executor_bindings.is_empty());
    assert!(facts.calls.is_empty());
}

#[test]
fn is_database_call_and_bindings_from_program() {
    let allocator = Allocator::default();
    let source = "import { query } from '@example/db'\nquery('SELECT 1')\nclient.query('SELECT 2')\nfoo.bar('no')\n";
    let parsed = crate::ast::parse(Path::new("db.ts"), &allocator, source, SourceType::ts());
    let bindings = executor_bindings(
        &parsed.program,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert!(bindings.contains("query"));
    assert!(!bindings.contains(super::super::bindings::MEMBER_QUERY_OPT_IN));
    let empty = HashSet::new();
    let mut saw_member = false;
    for statement in &parsed.program.body {
        if let oxc_ast::ast::Statement::ExpressionStatement(expr) = statement {
            if let Expression::CallExpression(call) = &expr.expression {
                if call.callee.is_member_expression() && is_database_call(call, &bindings) {
                    saw_member = true;
                    assert!(!is_database_call(call, &empty));
                }
            }
        }
    }
    assert!(!saw_member);
}

#[test]
fn parse_expr_helper_keeps_allocator_alive() {
    let (allocator, source) = parse_expr("'x'");
    let parsed = crate::ast::parse(Path::new("e.ts"), &allocator, &source, SourceType::ts());
    assert!(sql_text(first_init(&parsed.program)).is_some());
}

#[test]
fn type_only_and_empty_imports_do_not_bind() {
    let source = "import type { query } from '@example/db'\nimport { type read } from '@example/db'\nimport '@example/db'\nquery('SELECT 1')\n";
    let facts = extract_embedded_sql_from_source(
        Path::new("types.ts"),
        source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert!(facts.executor_bindings.is_empty());
    assert!(facts.calls.is_empty());
}

#[test]
fn spread_and_non_query_members_are_ignored() {
    let source = "import { query } from '@example/db'\nquery(...sql)\nfoo.bar('no')\nclient['other']('no')\nclient[key]('no')\n";
    let facts = extract_embedded_sql_from_source(
        Path::new("spread.ts"),
        source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 1);
    assert!(facts.calls[0].sql_text.is_none());
}

#[test]
fn computed_template_query_key_is_detected() {
    let source = "const client = { query: (sql: string) => sql }\nclient[`query`]('SELECT 9')\n";
    let facts = extract_embedded_sql_from_source(
        Path::new("computed.ts"),
        source,
        &EmbeddedSqlOptions::configured("", &["query".into()]),
    );
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 9"));
}

#[test]
fn unknown_extension_falls_back_to_typescript() {
    let facts = extract_embedded_sql_from_source(
        Path::new("no-extension"),
        "import { query } from '@example/db'\nquery('SELECT 5')\n",
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 5"));
}

#[test]
fn destructured_bindings_and_exported_consts_are_recorded() {
    let source = "import { query } from '@example/db'\nexport const q = 'SELECT exported'\nconst { skipped } = { skipped: 'no' }\nquery(q)\n";
    let facts = extract_embedded_sql_from_source(
        Path::new("export.ts"),
        source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT exported"));
}

#[test]
fn extract_from_program_matches_source_entry() {
    let allocator = Allocator::default();
    let path = Path::new("prog.ts");
    let source = "import { write } from '@example/db'\nwrite('SELECT 8')\n";
    let parsed = crate::ast::parse(path, &allocator, source, SourceType::ts());
    let facts = super::super::extract_embedded_sql_from_program(
        path,
        &parsed.program,
        source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.executor_bindings, ["write"]);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 8"));
}

#[test]
fn immutable_const_sql_is_analyzable() {
    let facts = extract("immutable-local.ts");
    assert_eq!(
        facts.calls[0].kind,
        super::super::EmbeddedSqlKind::ImmutableLocal
    );
    assert!(facts.calls[0]
        .sql_text
        .as_deref()
        .unwrap()
        .contains("topics"));
}

#[test]
fn static_concat_is_composed() {
    let facts = extract("composed-concat.ts");
    assert_eq!(facts.calls[0].kind, super::super::EmbeddedSqlKind::Composed);
    assert_eq!(
        facts.calls[0].sql_text.as_deref(),
        Some("SELECT id FROM topics")
    );
}
