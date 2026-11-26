use std::path::Path;

fn analyze(source: &str) -> crate::module_facts::TypeScriptModuleFacts {
    crate::ast::with_program(Path::new("rest.ts"), source, |program, _| {
        crate::module_facts::extract_with_arguments(program, true)
    })
    .unwrap()
}

#[test]
fn object_rest_handles_non_rest_and_non_declarator_bindings() {
    let source = r#"
        function f(opts: any) {
            const { cwd } = opts;
            const { a, ...rest } = opts;
            function inner() {}
            spawn("ls", { ...cwd, ...inner, ...rest, cwd: "/tmp" });
            spawn("ls", { cwd: rest.cwd, env: inner.env });
        }
        function g({ a, ...rest }: any = { cwd: "/a" }) {
            spawn("ls", { cwd: rest.cwd });
        }
        function h({ a }: any) {
            spawn("ls", { cwd: a.cwd });
        }
    "#;
    let facts = analyze(source);
    assert!(facts.call_arguments.is_some());
}
