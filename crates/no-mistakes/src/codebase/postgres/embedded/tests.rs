use super::{
    executed_query_text, executor_bindings, extract_embedded_sql_from_source, is_database_call,
    sql_text, EmbeddedSqlOptions,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_span::SourceType;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).expect("fixture")
}

fn extract(name: &str) -> super::EmbeddedSqlFileFacts {
    extract_embedded_sql_from_source(
        &fixture(name),
        &read_fixture(name),
        &EmbeddedSqlOptions::default(),
    )
}

fn parse_expr(source: &str) -> (Allocator, String) {
    (Allocator::default(), format!("const value = {source};"))
}

fn first_init<'a>(program: &'a oxc_ast::ast::Program<'a>) -> &'a Expression<'a> {
    match &program.body[0] {
        oxc_ast::ast::Statement::VariableDeclaration(declaration) => {
            declaration.declarations[0].init.as_ref().expect("init")
        }
        _ => panic!("expected variable"),
    }
}

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
    assert_eq!(by_sql("SELECT 3").callee, "query");
    assert_eq!(by_sql("SELECT 4").callee, "query");
    let shadowed = facts
        .calls
        .iter()
        .find(|call| call.callee == "query" && call.sql_text.is_none())
        .expect("param shadow");
    assert!(shadowed.line > 1);
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
fn custom_specifier_and_executor_names() {
    let source = "import { run as r, withTransaction } from '@app/db'\nrun('no')\nr('yes')\n";
    let options = EmbeddedSqlOptions {
        import_specifier: "@app/db".to_string(),
        executor_names: vec!["run".to_string()],
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
    let source = "import db, * as all from '@data-stores/psql'\ndb('x')\nall.query('y')\n";
    let facts = extract_embedded_sql_from_source(
        Path::new("ns.ts"),
        source,
        &EmbeddedSqlOptions::default(),
    );
    assert!(facts.executor_bindings.is_empty());
    assert_eq!(facts.calls.len(), 1);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("y"));
}

#[test]
fn is_database_call_and_bindings_from_program() {
    let allocator = Allocator::default();
    let source = "import { query } from '@data-stores/psql'\nquery('SELECT 1')\nfoo.bar('no')\n";
    let parsed = crate::ast::parse(Path::new("db.ts"), &allocator, source, SourceType::ts());
    let bindings = executor_bindings(&parsed.program, &EmbeddedSqlOptions::default());
    assert!(bindings.contains("query"));
    let empty = HashSet::new();
    for statement in &parsed.program.body {
        if let oxc_ast::ast::Statement::ExpressionStatement(expr) = statement {
            if let Expression::CallExpression(call) = &expr.expression {
                if is_database_call(call, &bindings) {
                    assert!(!is_database_call(call, &empty) || call.callee.is_member_expression());
                }
            }
        }
    }
}

#[test]
fn parse_expr_helper_keeps_allocator_alive() {
    let (allocator, source) = parse_expr("'x'");
    let parsed = crate::ast::parse(Path::new("e.ts"), &allocator, &source, SourceType::ts());
    assert!(sql_text(first_init(&parsed.program)).is_some());
}

#[test]
fn type_only_and_empty_imports_do_not_bind() {
    let source = "import type { query } from '@data-stores/psql'\nimport { type read } from '@data-stores/psql'\nimport '@data-stores/psql'\nquery('SELECT 1')\n";
    let facts = extract_embedded_sql_from_source(
        Path::new("types.ts"),
        source,
        &EmbeddedSqlOptions::default(),
    );
    assert!(facts.executor_bindings.is_empty());
    assert!(facts.calls.is_empty());
}

#[test]
fn spread_and_non_query_members_are_ignored() {
    let source = "import { query } from '@data-stores/psql'\nquery(...sql)\nfoo.bar('no')\nclient['other']('no')\nclient[key]('no')\n";
    let facts = extract_embedded_sql_from_source(
        Path::new("spread.ts"),
        source,
        &EmbeddedSqlOptions::default(),
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
        &EmbeddedSqlOptions::default(),
    );
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 9"));
}

#[test]
fn unknown_extension_falls_back_to_typescript() {
    let facts = extract_embedded_sql_from_source(
        Path::new("no-extension"),
        "import { query } from '@data-stores/psql'\nquery('SELECT 5')\n",
        &EmbeddedSqlOptions::default(),
    );
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 5"));
}

#[test]
fn destructured_bindings_and_exported_consts_are_recorded() {
    let source = "import { query } from '@data-stores/psql'\nexport const q = 'SELECT exported'\nconst { skipped } = { skipped: 'no' }\nquery(q)\n";
    let facts = extract_embedded_sql_from_source(
        Path::new("export.ts"),
        source,
        &EmbeddedSqlOptions::default(),
    );
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT exported"));
}

#[test]
fn extract_from_program_matches_source_entry() {
    let allocator = Allocator::default();
    let path = Path::new("prog.ts");
    let source = "import { write } from '@data-stores/psql'\nwrite('SELECT 8')\n";
    let parsed = crate::ast::parse(path, &allocator, source, SourceType::ts());
    let facts = super::extract_embedded_sql_from_program(
        path,
        &parsed.program,
        source,
        &EmbeddedSqlOptions::default(),
    );
    assert_eq!(facts.executor_bindings, ["write"]);
    assert_eq!(facts.calls[0].sql_text.as_deref(), Some("SELECT 8"));
}
