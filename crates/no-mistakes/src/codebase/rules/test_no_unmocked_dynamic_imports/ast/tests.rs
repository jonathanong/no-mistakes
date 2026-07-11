use super::*;

#[test]
fn extracts_dynamic_imports_and_mock_calls() {
    let source = r#"
vi.mock('./a.mts')
jest.unstable_mockModule(`./b.mts`, () => ({}))
await import('./a.mts')
await import(name)
"#;
    let facts = extract(Path::new("x.test.mts"), source).unwrap();
    assert_eq!(facts.mock_specifiers, vec!["./a.mts", "./b.mts"]);
    assert_eq!(facts.dynamic_imports.len(), 2);
    assert_eq!(
        facts.dynamic_imports[0].specifier.as_deref(),
        Some("./a.mts")
    );
    assert_eq!(facts.dynamic_imports[1].specifier, None);
}

#[test]
fn ignores_non_static_or_non_framework_mock_calls() {
    let source = r#"
foo.mock('./ignored.mts')
vi.mock()
vi.mock(name)
await import(`./${name}.mts`)
"#;
    let facts = extract(Path::new("x.test.mts"), source).unwrap();
    assert!(facts.mock_specifiers.is_empty());
    assert_eq!(facts.dynamic_imports[0].specifier, None);
}

#[test]
fn rejects_unsupported_file_extensions() {
    let Err(err) = extract(Path::new("x.txt"), "") else {
        panic!("unsupported file should fail");
    };
    assert!(err
        .to_string()
        .contains("unsupported JavaScript/TypeScript file"));
}

#[test]
fn recognizes_typed_mock_import_specifiers() {
    // Vitest/Jest typed mock specifiers: `import("./dep")` used as the first argument is a
    // type carrier for the mocked module's shape, not a runtime dynamic import. See #506.
    let source = r#"
vi.mock(import('./a.mts'), () => ({}))
vi.doMock(import('./b.mts'), () => ({}))
"#;
    let facts = extract(Path::new("x.test.mts"), source).unwrap();
    assert_eq!(facts.mock_specifiers, vec!["./a.mts", "./b.mts"]);
    assert!(
        facts.dynamic_imports.is_empty(),
        "type-carrier imports must not be treated as dynamic imports"
    );
}

#[test]
fn keeps_dynamic_imports_inside_mock_factories() {
    // The type-carrier import (1st arg) is excluded, but a genuine `import(...)` written
    // inside the factory (2nd arg) must still be discovered and checked.
    let source = r#"
vi.mock(import('./a.mts'), () => import('./real.mts'))
"#;
    let facts = extract(Path::new("x.test.mts"), source).unwrap();
    assert_eq!(facts.mock_specifiers, vec!["./a.mts"]);
    assert_eq!(facts.dynamic_imports.len(), 1);
    assert_eq!(
        facts.dynamic_imports[0].specifier.as_deref(),
        Some("./real.mts")
    );
}

#[test]
fn excludes_non_static_typed_mock_carrier() {
    // A non-static import specifier (`import(name)`) as the first argument still marks the
    // carrier as a type carrier (excluded from dynamic_imports), but yields no mock
    // specifier since the module path is not statically known.
    let source = r#"
vi.mock(import(name), () => ({}))
"#;
    let facts = extract(Path::new("x.test.mts"), source).unwrap();
    assert!(facts.mock_specifiers.is_empty());
    assert!(facts.dynamic_imports.is_empty());
}
