use super::in_named_branch;
use oxc_ast::{ast::Expression, AstKind};
use oxc_semantic::SemanticBuilder;
use std::path::Path;

fn is_hit_in_named_branch(source: &str) -> bool {
    crate::ast::with_program(Path::new("branch.ts"), source, |program, _| {
        let semantic = SemanticBuilder::new()
            .with_build_nodes(true)
            .build(program)
            .semantic;
        let hit = semantic
            .nodes()
            .iter()
            .find(|node| {
                matches!(
                    node.kind(),
                    AstKind::CallExpression(call)
                        if matches!(call.callee.get_inner_expression(), Expression::Identifier(id) if id.name == "hit")
                )
            })
            .expect("hit call")
            .id();
        in_named_branch(semantic.nodes(), hit)
    })
    .unwrap()
}

#[test]
fn branch_arm_handles_parenthesized_ternary_iife_and_loops() {
    assert!(is_hit_in_named_branch(
        "cond1 ? (cond2 ? hit() : other()) : other2();"
    ));
    assert!(is_hit_in_named_branch(
        "cond ? (() => { hit(); })() : null;"
    ));
    assert!(!is_hit_in_named_branch("while (cond) { hit(); }"));
    assert!(is_hit_in_named_branch("if (cond) { hit(); }"));
}
