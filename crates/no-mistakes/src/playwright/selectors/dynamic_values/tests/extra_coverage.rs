use super::super::*;
use super::parse_and_collect;
use crate::playwright::ast;
use crate::playwright::test_support::fixture_path;
use std::path::Path;

fn page_extras_path() -> std::path::PathBuf {
    fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page-extras.tsx",
    ])
}

fn resolve_from_extras(local_name: &str) -> Vec<String> {
    let page_path = page_extras_path();
    let source = std::fs::read_to_string(&page_path).unwrap();
    ast::with_program(&page_path, &source, |program, _| {
        super::super::cross_file::resolve_imported_values(local_name, program, &page_path)
    })
    .unwrap()
}

#[test]
fn cross_file_resolves_default_fn_export_with_if_else_body() {
    let mut values = resolve_from_extras("DefaultFn");
    values.sort();
    assert_eq!(values, vec!["default-fn-else", "default-fn-val"]);
}

#[test]
fn cross_file_resolves_default_obj_export() {
    let mut values = resolve_from_extras("DefaultObj");
    values.sort();
    assert_eq!(values, vec!["obj-a-val", "obj-b-val"]);
}

#[test]
fn cross_file_resolves_default_arrow_expression_body() {
    let values = resolve_from_extras("DefaultArrowExpr");
    assert_eq!(values, vec!["arrow-expr-val"]);
}

#[test]
fn cross_file_resolves_default_arrow_block_body() {
    let values = resolve_from_extras("DefaultArrowBlock");
    assert_eq!(values, vec!["arrow-block-val"]);
}

#[test]
fn cross_file_default_literal_export_returns_empty() {
    let values = resolve_from_extras("DefaultLiteral");
    assert!(values.is_empty());
}

#[test]
fn cross_file_namespace_import_returns_empty() {
    let values = resolve_from_extras("Ns");
    assert!(values.is_empty());
}

#[test]
fn cross_file_fn_with_expr_statement_consequent() {
    let values = resolve_from_extras("fnWithExprConsequent");
    assert_eq!(values, vec!["expr-base-val"]);
}

#[test]
fn cross_file_uninitialized_export_let_returns_empty() {
    let values = resolve_from_extras("uninitializedLet");
    assert!(values.is_empty());
}

#[test]
fn cross_file_array_destructured_export_returns_empty() {
    let values = resolve_from_extras("firstArr");
    assert!(values.is_empty());
}

#[test]
fn cross_file_unresolvable_relative_import_returns_empty() {
    let source = r#"import { x } from './definitely-does-not-exist-xyz';"#;
    let values = ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        super::super::cross_file::resolve_imported_values("x", program, Path::new("fixture.tsx"))
    })
    .unwrap();
    assert!(values.is_empty());
}

#[test]
fn visitor_collect_from_statements_with_no_scope_returns_empty() {
    let source = r#"const x = 'hello';"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        let mut visitor = super::super::visitor::DynamicValuesVisitor::new();
        visitor.collect_from_statements(&program.body);
        assert!(visitor.collected.is_empty());
    })
    .unwrap();
}

#[test]
fn visitor_else_branch_adds_new_variable_to_all_names() {
    let source = r#"
function Page() {
  let a, b;
  if (cond) {
    a = 'val-a';
  } else {
    b = 'val-b';
  }
}
"#;
    let collected = parse_and_collect(source);
    let a_entry = collected.iter().find(|e| e.name == "a").expect("a entry");
    assert_eq!(a_entry.values, vec!["val-a"]);
    let b_entry = collected.iter().find(|e| e.name == "b").expect("b entry");
    assert_eq!(b_entry.values, vec!["val-b"]);
}
