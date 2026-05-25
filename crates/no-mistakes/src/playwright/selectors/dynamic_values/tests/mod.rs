use super::collect::collect_object_string_values;
use super::*;
use crate::playwright::ast;
use std::path::Path;

pub(super) mod coverage;
pub(super) mod extra_coverage;

pub(super) fn parse_and_collect(source: &str) -> Vec<DynamicIdentifierValues> {
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
    assert_eq!(entries.len(), 2);
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
