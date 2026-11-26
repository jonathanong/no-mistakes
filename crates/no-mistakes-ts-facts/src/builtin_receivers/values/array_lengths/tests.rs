use super::result_length;
use oxc_ast::ast::{Expression, Statement};
use std::path::Path;

fn eval(call_src: &str, method: &str, length: Option<usize>, empty: bool) -> Option<usize> {
    crate::ast::with_program(Path::new("arr.ts"), call_src, |program, _| {
        let Statement::ExpressionStatement(stmt) = &program.body[0] else {
            panic!("expected expression statement");
        };
        let Expression::CallExpression(call) = &stmt.expression else {
            panic!("expected call expression");
        };
        result_length(method, length, empty, call)
    })
    .unwrap()
}

#[test]
fn array_lengths_cover_zero_filter_unknown_slice_and_spread_splice() {
    assert_eq!(eval("a.filter(fn);", "filter", Some(0), false), Some(0));
    assert_eq!(eval("a.slice(0, x);", "slice", Some(3), false), None);
    assert_eq!(eval("a.splice(0, 1);", "splice", None, false), None);
    assert_eq!(eval("a.splice(...args);", "splice", Some(3), false), None);
    assert_eq!(eval("a.splice(0, +2);", "splice", Some(3), false), Some(2));
    assert_eq!(eval("a.splice(0, -1);", "splice", Some(3), false), Some(0));
    assert_eq!(eval("a.splice(0, ~1);", "splice", Some(3), false), None);
}
