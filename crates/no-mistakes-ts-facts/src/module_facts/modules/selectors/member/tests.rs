use super::{body, read};
use oxc_ast::ast::Statement;
use std::path::Path;

fn eval_read(expr: &str) -> Option<String> {
    let source = format!("({expr});");
    crate::ast::with_program(Path::new("member.ts"), &source, |program, _| {
        let Statement::ExpressionStatement(stmt) = &program.body[0] else {
            panic!("expected expression statement");
        };
        read(&stmt.expression, "m")
    })
    .unwrap()
}

fn eval_body(expr: &str) -> Option<String> {
    let source = format!("({expr});");
    crate::ast::with_program(Path::new("member.ts"), &source, |program, _| {
        let Statement::ExpressionStatement(stmt) = &program.body[0] else {
            panic!("expected expression statement");
        };
        body(&stmt.expression, "m")
    })
    .unwrap()
}

#[test]
fn member_read_and_body_handle_static_computed_and_rejected_shapes() {
    assert_eq!(eval_read("m.Name").as_deref(), Some("Name"));
    assert_eq!(eval_read(r#"m["Name"]"#).as_deref(), Some("Name"));
    assert_eq!(eval_read("m[key]"), None);
    assert_eq!(eval_read("m?.Name"), None);
    assert_eq!(eval_read("other.Name"), None);
    assert_eq!(eval_read("foo()"), None);
    assert_eq!(eval_read("get().Name"), None);

    assert_eq!(eval_body("m.Name").as_deref(), Some("Name"));
    assert_eq!(eval_body("{ default: m.Name }").as_deref(), Some("Name"));
    assert_eq!(eval_body("{ other: m.Name }"), None);
    assert_eq!(eval_body("{ [k]: m.Name }"), None);
    assert_eq!(eval_body("{ ...x }"), None);
    assert_eq!(eval_body("{ default() { return m.Name; } }"), None);
}
