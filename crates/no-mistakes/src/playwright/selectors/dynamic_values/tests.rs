use super::collect::{
    call_identifier_name, collect_assignments_from_stmt, collect_object_string_values,
    collect_returns_from_statements, extract_computed_member_object_name,
};
use super::*;
use crate::playwright::ast;
use std::path::Path;

fn parse_and_collect(source: &str) -> Vec<DynamicIdentifierValues> {
    ast::with_program(Path::new("fixture.tsx"), source, |program, src| {
        collect_dynamic_identifier_values(program, src)
    })
    .unwrap()
}

#[test]
fn collect_string_leaves_string_literal() {
    let source = r#"const x = 'hello';"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let leaves = collect_string_leaves(init);
                        assert_eq!(leaves, vec!["hello"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_string_leaves_ternary() {
    let source = r#"const x = cond ? 'a' : 'b';"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let mut leaves = collect_string_leaves(init);
                        leaves.sort();
                        assert_eq!(leaves, vec!["a", "b"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_string_leaves_logical() {
    let source = r#"const x = 'a' || 'b';"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let mut leaves = collect_string_leaves(init);
                        leaves.sort();
                        assert_eq!(leaves, vec!["a", "b"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_string_leaves_non_string() {
    let source = r#"const x = 1 + 2;"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let leaves = collect_string_leaves(init);
                        assert!(leaves.is_empty());
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_object_string_values_basic() {
    let source = r#"const x = { a: 'val-a', b: 'val-b' };"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let mut values = collect_object_string_values(init);
                        values.sort();
                        assert_eq!(values, vec!["val-a", "val-b"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_object_string_values_skips_computed() {
    let source = r#"const x = { [key]: 'val-computed', a: 'val-static' };"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let values = collect_object_string_values(init);
                        assert_eq!(values, vec!["val-static"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_function_return_strings_basic() {
    let source = r#"
function getSelector(cond) {
  if (cond) return 'fn-a';
  return 'fn-b';
}
"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        let mut values = collect_function_return_strings("getSelector", program);
        values.sort();
        assert_eq!(values, vec!["fn-a", "fn-b"]);
    })
    .unwrap();
}

#[test]
fn collect_function_return_strings_missing() {
    let source = r#"function other() { return 'x'; }"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        let values = collect_function_return_strings("notfound", program);
        assert!(values.is_empty());
    })
    .unwrap();
}

#[test]
fn resolve_dynamic_identifier_ternary() {
    let source = r#"
function Page() {
  const dataPw = cond ? 'option-a' : 'option-b';
  return dataPw;
}
"#;
    let collected = parse_and_collect(source);
    assert!(!collected.is_empty());
    // Find the span of `dataPw` usage inside the function body
    let entry = collected
        .iter()
        .find(|e| e.name == "dataPw")
        .expect("dataPw entry");
    let mut values = entry.values.clone();
    values.sort();
    assert_eq!(values, vec!["option-a", "option-b"]);
}

#[test]
fn resolve_dynamic_identifier_scope_tightest() {
    let source = r#"
function Outer() {
  const x = cond ? 'outer-a' : 'outer-b';
  function Inner() {
    const x = cond ? 'inner-a' : 'inner-b';
    return x;
  }
  return x;
}
"#;
    let collected = parse_and_collect(source);
    let entries: Vec<_> = collected.iter().filter(|e| e.name == "x").collect();
    // There should be two entries for x, one in each scope
    assert_eq!(entries.len(), 2);
    // Tightest scope has smaller span
    let smallest = entries
        .iter()
        .min_by_key(|e| e.scope.end - e.scope.start)
        .unwrap();
    let mut values = smallest.values.clone();
    values.sort();
    assert_eq!(values, vec!["inner-a", "inner-b"]);
}

#[test]
fn resolve_dynamic_identifier_if_else() {
    let source = r#"
function Page() {
  let dataPw;
  if (cond) {
    dataPw = 'branch-a';
  } else {
    dataPw = 'branch-b';
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
    assert_eq!(values, vec!["branch-a", "branch-b"]);
}

// ── collect_string_leaves TS wrapper branches ────────────────────────────────

#[test]
fn collect_string_leaves_ts_as_expression() {
    let source = r#"const x = 'hello' as string;"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let leaves = collect_string_leaves(init);
                        assert_eq!(leaves, vec!["hello"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_string_leaves_ts_non_null_expression() {
    let source = r#"const x = 'hello'!;"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let leaves = collect_string_leaves(init);
                        assert_eq!(leaves, vec!["hello"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_string_leaves_ts_satisfies_expression() {
    let source = r#"const x = 'hello' satisfies string;"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let leaves = collect_string_leaves(init);
                        assert_eq!(leaves, vec!["hello"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_string_leaves_parenthesized_expression() {
    let source = r#"const x = ('hello');"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let leaves = collect_string_leaves(init);
                        assert_eq!(leaves, vec!["hello"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

// ── collect_object_string_values TSAsExpression branch ───────────────────────

#[test]
fn collect_object_string_values_ts_as_expression() {
    let source = r#"const x = { a: 'val-a', b: 'val-b' } as const;"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let mut values = collect_object_string_values(init);
                        values.sort();
                        assert_eq!(values, vec!["val-a", "val-b"]);
                    }
                }
            }
        }
    })
    .unwrap();
}

#[test]
fn collect_object_string_values_ts_as_non_object_returns_empty() {
    // TSAsExpression where inner expression is NOT an object → returns empty
    let source = r#"const x = someVar as const;"#;
    ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        for stmt in &program.body {
            if let oxc_ast::ast::Statement::VariableDeclaration(decl) = stmt {
                for d in &decl.declarations {
                    if let Some(init) = d.init.as_ref() {
                        let values = collect_object_string_values(init);
                        assert!(values.is_empty());
                    }
                }
            }
        }
    })
    .unwrap();
}

// ── collect_returns_from_statements: else block (block return) ────────────────

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

// ── collect_returns_from_statements: block statement wrapping ────────────────

#[test]
fn collect_returns_from_statements_block_wrapper() {
    // covers the BlockStatement arm in collect_returns_from_statements
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

// ── collect_returns_from_stmt: block return (private fn via public wrapper) ──

#[test]
fn collect_returns_from_statements_block_return_in_if() {
    // covers the BlockStatement arm in collect_returns_from_stmt
    // (if-consequent is a block containing a return)
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

// ── call_identifier_name: non-identifier callee ──────────────────────────────

#[test]
fn call_identifier_name_non_identifier_returns_none() {
    // obj.method() → callee is a static member expression, not an Identifier
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

// ── extract_computed_member_object_name ──────────────────────────────────────

#[test]
fn extract_computed_member_object_name_non_identifier_object_returns_none() {
    // fn()[key] → object is a call expression, not an identifier
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
    // map[key] as any → TSAsExpression wrapping ComputedMemberExpression with identifier object
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
    // TSAsExpression wrapping a static member expression (not computed) → None
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
    // fn()[key] as any → TSAsExpression wrapping ComputedMember with non-identifier object
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

// ── collect_assignments_from_stmt: non-string right side ────────────────────

#[test]
fn collect_assignments_from_stmt_non_string_right_side() {
    // x = 42 → right side is numeric literal, not a string → collector is never called
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
    // obj.x = 'val' → left side is not an identifier → collector is never called
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

// ── visitor: switch statement collection ────────────────────────────────────

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

// ── visitor: object map with no string values ────────────────────────────────

#[test]
fn resolve_dynamic_identifier_object_map_no_string_values() {
    // Object has only numeric values → no object_map entry, so computed member
    // resolves to the __obj__ sentinel which becomes empty after resolution
    let source = r#"
function Page(key) {
  const map = { a: 1, b: 2 };
  const x = map[key];
  return x;
}
"#;
    let collected = parse_and_collect(source);
    // x should not appear in collected since its values would be empty after resolution
    let entry = collected.iter().find(|e| e.name == "x");
    assert!(entry.is_none());
}

// ── visitor: top-level call (no current scope) ───────────────────────────────

#[test]
fn collect_dynamic_identifier_values_no_scope_at_top_level() {
    // collect_from_statements is only called when inside a function scope.
    // A top-level variable declaration is never processed by collect_from_statements,
    // so there are no dynamic identifier values for top-level code.
    let source = r#"const x = cond ? 'a' : 'b';"#;
    let collected = parse_and_collect(source);
    assert!(collected.is_empty());
}

// ── visitor: function call with no same-file match → sentinel becomes empty ──

#[test]
fn resolve_dynamic_identifier_call_no_same_file_match() {
    // dataPw = unknownFn() → __call__unknownFn sentinel
    // collect_function_return_strings("unknownFn") finds nothing → empty
    // No file path given (parse_and_collect uses Path::new("fixture.tsx"))
    // → entry should be absent since values are empty after resolution
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

// ── cross_file: resolve_imported_values ─────────────────────────────────────

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
        super::cross_file::resolve_imported_values("CONST_SELECTOR", program, &page_path)
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
        super::cross_file::resolve_imported_values("getSelector", program, &page_path)
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
        super::cross_file::resolve_imported_values("selectorMap", program, &page_path)
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
        super::cross_file::resolve_imported_values("NONEXISTENT", program, &page_path)
    })
    .unwrap();
    assert!(values.is_empty());
}

#[test]
fn cross_file_non_relative_import_returns_empty() {
    // Importing from a package (not a relative path) → resolve_import returns None → empty
    let source = r#"import { x } from 'some-pkg';"#;
    let values = ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        super::cross_file::resolve_imported_values("x", program, Path::new("fixture.tsx"))
    })
    .unwrap();
    assert!(values.is_empty());
}

#[test]
fn cross_file_default_import_specifier_returns_empty_for_no_default_export() {
    // Default import: import Foo from './missing-module'
    // resolve_import will fail to find the file → returns empty
    let source = r#"import Foo from './missing-module';"#;
    let values = ast::with_program(Path::new("fixture.tsx"), source, |program, _| {
        super::cross_file::resolve_imported_values("Foo", program, Path::new("fixture.tsx"))
    })
    .unwrap();
    assert!(values.is_empty());
}
