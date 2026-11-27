use super::immediate_read;
use oxc_ast::{ast::Expression, AstKind};
use oxc_semantic::SemanticBuilder;
use std::path::Path;

#[test]
fn local_env_ignores_spread_registration_and_non_identifier_stub_objects() {
    let source = r#"
        import { vi } from "vitest";
        test("spread", ...args);
        test("non-ident", () => {
            get().stubEnv("A", "1");
            run();
        });
    "#;
    let facts = crate::ast::with_program(Path::new("env.test.ts"), source, |program, _| {
        crate::module_facts::extract_with_arguments(program, true)
    })
    .unwrap();
    assert!(facts.call_arguments.is_some());
}

#[test]
fn immediate_read_handles_non_call_missing_arg_and_nested_return() {
    let source = r#"
        function noArg() {
            return run();
        }
        function nestedReturn() {
            if (ok) {
                return run(process.env.X);
            }
        }
        function directRead() {
            return run(process.env.X ?? "default");
        }
    "#;
    crate::ast::with_program(Path::new("immediate.ts"), source, |program, _| {
        let semantic = SemanticBuilder::new()
            .with_build_nodes(true)
            .build(program)
            .semantic;
        let root_id = semantic.nodes().iter().next().expect("program node").id();
        assert!(!immediate_read(&semantic, root_id));

        let call_results: Vec<bool> = semantic
            .nodes()
            .iter()
            .filter(|node| {
                matches!(
                    node.kind(),
                    AstKind::CallExpression(call)
                        if matches!(call.callee.get_inner_expression(), Expression::Identifier(id) if id.name == "run")
                )
            })
            .map(|node| immediate_read(&semantic, node.id()))
            .collect();
        assert_eq!(call_results, vec![false, false, true]);
    })
    .unwrap();
}
