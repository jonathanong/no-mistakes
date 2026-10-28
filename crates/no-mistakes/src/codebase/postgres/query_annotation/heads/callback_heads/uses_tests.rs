use super::{uses::Uses, Payload};
use oxc_ast::ast::{Declaration, Statement};
use std::collections::BTreeMap;

#[test]
fn callback_producer_requires_closed_receiver_and_payload_uses() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads/callback-producer-uses.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let summaries = crate::ast::with_program(&path, &source, |program, _| {
        program
            .body
            .iter()
            .filter_map(|statement| {
                let Statement::ExportDeclaration(export) = statement else {
                    return None;
                };
                let Declaration::FunctionDeclaration(function) = &export.declaration else {
                    return None;
                };
                let name = function.id.as_ref()?.name.as_str().to_string();
                let body = function.body.as_ref()?;
                let proof = Uses::inspect(body, vec!["base", "execute"], 1);
                Some((name, proof))
            })
            .collect::<BTreeMap<_, _>>()
    })
    .unwrap();
    assert_eq!(summaries.len(), 9);
    assert_eq!(summaries["safe"], Some(Payload::Parameter(0)));
    for name in [
        "optional",
        "spread",
        "dynamicScope",
        "argumentsAlias",
        "shadowed",
        "deleted",
        "nestedClass",
        "oversized",
    ] {
        assert_eq!(summaries[name], None, "unsafe callback source: {name}");
    }
}
