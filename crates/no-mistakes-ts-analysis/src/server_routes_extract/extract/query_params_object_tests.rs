use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn expression_matches_request_object(source: &str) -> bool {
    let allocator = Allocator::default();
    let source = format!("const value = {source};");
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[0] else {
        panic!("expected variable declaration");
    };
    let expression = declaration.declarations[0].init.as_ref().unwrap();
    is_request_query_object(expression)
}

fn expression_matches_query_object(source: &str) -> bool {
    let allocator = Allocator::default();
    let source = format!("const value = {source};");
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[0] else {
        panic!("expected variable declaration");
    };
    let expression = declaration.declarations[0].init.as_ref().unwrap();
    expression_is_query_object(expression, &BTreeSet::new())
}

fn expression_matches_request_object_at_nesting(source: &str, nesting: u8) -> bool {
    let allocator = Allocator::default();
    let source = format!("const value = {source};");
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[0] else {
        panic!("expected variable declaration");
    };
    let expression = declaration.declarations[0].init.as_ref().unwrap();
    is_request_object_expr(expression, nesting)
}

#[test]
fn request_query_object_detection_handles_optional_and_nested_members() {
    assert!(expression_matches_query_object("req?.query"));
    assert!(!expression_matches_query_object("db?.query"));
    assert!(expression_matches_request_object("context?.request"));
    assert!(!expression_matches_request_object("context?.[dynamicKey]"));
    assert!(!expression_matches_request_object("context?.other"));
    assert!(!expression_matches_request_object(
        "context?.request?.query()"
    ));
    assert!(expression_matches_request_object("context.req"));
    assert!(expression_matches_request_object_at_nesting(
        "context.req",
        1
    ));
    assert!(!expression_matches_request_object_at_nesting(
        "context.req",
        2
    ));
    assert!(!expression_matches_request_object_at_nesting("call()", 1));
    assert!(!expression_matches_request_object("context.deep.req"));
    assert!(!expression_matches_request_object(
        "context.req.ctx.request"
    ));
    assert!(!expression_matches_request_object("call().req"));
    assert!(!expression_matches_request_object("unrelated.req"));
    assert!(!expression_matches_request_object("123"));
}
