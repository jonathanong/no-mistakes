use super::{caller, producer};
use oxc_ast::ast::Function;
use oxc_ast_visit::{walk, Visit};
use std::collections::BTreeMap;

#[derive(Default)]
struct Shapes {
    producers: BTreeMap<String, Option<usize>>,
    callers: BTreeMap<String, usize>,
}
impl<'a> Visit<'a> for Shapes {
    fn visit_function(&mut self, value: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if let (Some(id), Some(body)) = (&value.id, &value.body) {
            self.producers.insert(
                id.name.to_string(),
                producer::function(value).map(|candidate| candidate.appends.len()),
            );
            self.callers.insert(
                id.name.to_string(),
                caller::callers(&body.statements, body.span).len(),
            );
        }
        walk::walk_function(self, value, flags);
    }
}
fn saved_shapes() -> Shapes {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("async-return-grammar/src/shapes.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let mut shapes = Shapes::default();
        shapes.visit_program(program);
        shapes
    })
    .unwrap()
}
#[test]
fn async_origin_grammar_producer_bounds_and_sequential_setup_shapes() {
    let shapes = saved_shapes();
    for (name, appends) in [
        ("setupValuesBeforeRoot", 0),
        ("rootDepthEight", 0),
        ("discardedDepthEight", 1),
    ] {
        assert_eq!(shapes.producers[name], Some(appends), "{name}");
    }
    for name in [
        "wrongTerminalStatement",
        "nonIdentifierTag",
        "nonNativeRoot",
        "emptyAppendRoot",
        "emptyAppendStatement",
        "rootDepthNine",
        "discardedDepthNine",
        "bareRootExpression",
    ] {
        assert_eq!(shapes.producers[name], None, "{name}");
    }
}
#[test]
fn async_origin_grammar_callers_reject_noncall_and_wrong_argument_shapes() {
    let shapes = saved_shapes();
    assert_eq!(shapes.callers["callerAccepted"], 1);
    for name in [
        "callerNonCallTerminal",
        "callerNonIdentifierArgument",
        "callerDifferentArgumentOwner",
    ] {
        assert_eq!(shapes.callers[name], 0, "{name}");
    }
}
