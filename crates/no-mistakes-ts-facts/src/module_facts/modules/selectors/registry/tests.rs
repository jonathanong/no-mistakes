use std::path::Path;

fn analyze(source: &str) -> crate::module_facts::TypeScriptModuleFacts {
    crate::ast::with_program(Path::new("registry.ts"), source, |program, _| {
        crate::module_facts::extract_with_arguments(program, false)
    })
    .unwrap()
}

#[test]
fn registry_selector_rejects_unproven_and_handles_unused_forms() {
    let unused = analyze("const unused = { a: () => import('./a') };");
    assert_eq!(unused.loads.len(), 1);
    assert_eq!(unused.dynamic_import_selectors.len(), 1);
    assert!(unused.dynamic_import_selectors[0].1.is_empty());

    let rejected = [
        "const k = 'a'; const comp = { [k]: () => import('./a') }; comp.a();",
        "const unawaited = { a: () => import('./a') }; unawaited.a();",
        "const opt = { a: () => import('./a') }; async function f() { await opt?.a(); }",
        "const args = { a: () => import('./a') }; async function f() { await args.a(1); }",
        "const refed = { a: () => import('./a') }; const x = refed.a; void x;",
        "const escaped = { a: () => import('./a') }; use(escaped);",
        "export const exp = { a: () => import('./a') }; async function f() { await exp.a(); }",
        "let mutReg = { a: () => import('./a') }; async function f() { await mutReg.a(); }",
        "const dup = { a: () => import('./a'), a: () => import('./b') }; async function f() { await dup.a(); }",
        "const spread = { ...base, a: () => import('./a') }; async function f() { await spread.a(); }",
        "const bad = { a: () => import('./a') }; async function f() { await bad.missing(); }",
    ];
    for source in rejected {
        let facts = analyze(source);
        assert!(!facts.loads.is_empty(), "{source}");
        assert!(facts.dynamic_import_selectors.is_empty(), "{source}");
    }
}
