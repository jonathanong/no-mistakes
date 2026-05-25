use super::super::collect::{
    call_identifier_name, collect_assignments_from_stmt, collect_returns_from_statements,
    extract_computed_member_object_name,
};
use super::super::*;
use super::parse_and_collect;
use crate::playwright::ast;
use std::path::Path;

#[test]
fn collect_function_return_strings_else_block() {
    let source = r#"
function fn1(c) {
  if (c) { return 'a'; } else { return 'b'; }
}
"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        let mut values = collect_function_return_strings("fn1", program);
        values.sort();
        assert_eq!(values, vec!["a", "b"]);
    })
    .unwrap();
}

#[test]
fn collect_returns_from_statements_block_wrapper() {
    let source = r#"
function fn2() {
  { return 'block-val'; }
}
"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        let mut values = collect_function_return_strings("fn2", program);
        values.sort();
        assert_eq!(values, vec!["block-val"]);
    })
    .unwrap();
}

#[test]
fn collect_returns_from_statements_block_return_in_if() {
    let source = r#"
function fn3(c) {
  if (c) { return 'if-block'; }
}
"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::FunctionDeclaration(f) = stmt {
                if f.id.as_ref().is_some_and(|id| id.name == "fn3") {
                    if let Some(body) = &f.body {
                        let mut values = Vec::new();
                        collect_returns_from_statements(&body.statements, &mut values);
                        assert_eq!(values, vec!["if-block"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn call_identifier_name_non_identifier_returns_none() {
    let source = r#"const x = obj.method();"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(oxc_ast::ast::Expression::CallExpression(call)) =
                        d.init.as_ref().map(|e| e)
                    {
                        let result = call_identifier_name(&call.callee);
                        assert!(result.is_none());
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn extract_computed_member_object_name_non_identifier_object_returns_none() {
    let source = r#"const x = fn()[key];"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let result = extract_computed_member_object_name(init);
                        assert!(result.is_none());
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn extract_computed_member_object_name_ts_as_wrapping_computed_member_with_identifier() {
    let source = r#"const x = map[key] as any;"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let result = extract_computed_member_object_name(init);
                        assert_eq!(result, Some("map"));
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn extract_computed_member_object_name_ts_as_non_computed_returns_none() {
    let source = r#"const x = map.key as any;"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let result = extract_computed_member_object_name(init);
                        assert!(result.is_none());
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn extract_computed_member_object_name_ts_as_non_identifier_object_returns_none() {
    let source = r#"const x = fn()[key] as any;"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let result = extract_computed_member_object_name(init);
                        assert!(result.is_none());
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_assignments_from_stmt_non_string_right_side() {
    let source = r#"function f() { x = 42; }"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::FunctionDeclaration(f) = stmt {
                if let Some(body) = &f.body {
                    for s in &body.statements {
                        let mut collected: Vec<(String, String)> = Vec::new();
                        collect_assignments_from_stmt(s, &mut |name, value| {
                            collected.push((name.to_string(), value.to_string()));
                        });
                        assert!(collected.is_empty());
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_assignments_from_stmt_non_identifier_target() {
    let source = r#"function f() { obj.x = 'val'; }"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::FunctionDeclaration(f) = stmt {
                if let Some(body) = &f.body {
                    for s in &body.statements {
                        let mut collected: Vec<(String, String)> = Vec::new();
                        collect_assignments_from_stmt(s, &mut |name, value| {
                            collected.push((name.to_string(), value.to_string()));
                        });
                        assert!(collected.is_empty());
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn resolve_dynamic_identifier_switch_statement() {
    let source = r#"
function Page(x) {
  let dataPw;
  switch(x) {
    case 'a': dataPw = 'switch-a'; break;
    case 'b': dataPw = 'switch-b'; break;
  }
  return dataPw;
}
"#;
    let collected = parse_and_collect(source);
    let entry = collected
        .iter()
        .find(|e| e.name == "dataPw")
        .expect("dataPw entry");
    let mut values = entry.values.clone();
    values.sort();
    assert_eq!(values, vec!["switch-a", "switch-b"]);
}

#[test]
fn resolve_dynamic_identifier_object_map_no_string_values() {
    let source = r#"
function Page(key) {
  const map = { a: 1, b: 2 };
  const x = map[key];
  return x;
}
"#;
    let collected = parse_and_collect(source);
    let entry = collected.iter().find(|e| e.name == "x");
    assert!(entry.is_none());
}

#[test]
fn collect_dynamic_identifier_values_no_scope_at_top_level() {
    let source = r#"const x = cond ? 'a' : 'b';"#;
    let collected = parse_and_collect(source);
    assert!(collected.is_empty());
}

#[test]
fn resolve_dynamic_identifier_call_no_same_file_match() {
    let source = r#"
function Page() {
  const dataPw = unknownFn();
  return dataPw;
}
"#;
    let collected = parse_and_collect(source);
    let entry = collected.iter().find(|e| e.name == "dataPw");
    assert!(entry.is_none());
}

#[test]
fn cross_file_resolves_named_export_const() {
    use crate::playwright::test_support::fixture_path;
    let page_path = fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page.tsx",
    ]);
    let source = std::fs::read_to_string(&page_path).unwrap();
    let values = ast::with_program(&page_path, &source, |program, _| {
        super::super::cross_file::resolve_imported_values("CONST_SELECTOR", program, &page_path)
    })
    .unwrap();
    assert_eq!(values, vec!["imported-const"]);
}

#[test]
fn cross_file_resolves_named_export_function() {
    use crate::playwright::test_support::fixture_path;
    let page_path = fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page.tsx",
    ]);
    let source = std::fs::read_to_string(&page_path).unwrap();
    let values = ast::with_program(&page_path, &source, |program, _| {
        super::super::cross_file::resolve_imported_values("getSelector", program, &page_path)
    })
    .unwrap();
    assert_eq!(values, vec!["imported-fn-val"]);
}

#[test]
fn cross_file_resolves_named_export_object_all_values() {
    use crate::playwright::test_support::fixture_path;
    let page_path = fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page.tsx",
    ]);
    let source = std::fs::read_to_string(&page_path).unwrap();
    let mut values = ast::with_program(&page_path, &source, |program, _| {
        super::super::cross_file::resolve_imported_values("selectorMap", program, &page_path)
    })
    .unwrap();
    values.sort();
    assert_eq!(values, vec!["imported-obj-a", "imported-obj-b"]);
}

#[test]
fn cross_file_non_existent_import_returns_empty() {
    use crate::playwright::test_support::fixture_path;
    let page_path = fixture_path(&[
        "ast-snippets",
        "selectors",
        "dynamic-cross-file",
        "page.tsx",
    ]);
    let source = std::fs::read_to_string(&page_path).unwrap();
    let values = ast::with_program(&page_path, &source, |program, _| {
        super::super::cross_file::resolve_imported_values("NONEXISTENT", program, &page_path)
    })
    .unwrap();
    assert!(values.is_empty());
}

#[test]
fn cross_file_non_relative_import_returns_empty() {
    let source = r#"import { x } from 'some-pkg';"#;
    let values = ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        super::super::cross_file::resolve_imported_values("x", program, Path::new("fixture.tsx"))
    })
    .unwrap();
    assert!(values.is_empty());
}

#[test]
fn cross_file_default_import_specifier_returns_empty_for_no_default_export() {
    let source = r#"import Foo from './missing-module';"#;
    let values = ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        super::super::cross_file::resolve_imported_values("Foo", program, Path::new("fixture.tsx"))
    })
    .unwrap();
    assert!(values.is_empty());
}
