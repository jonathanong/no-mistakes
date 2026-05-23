use super::helpers::{
    first_call_expression, first_statement_assignment_call_expression,
    object_argument_from_call_expression,
};

#[test]
#[should_panic]
fn test_first_call_expression_panics_when_not_expression_statement() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "if (true) {}";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    first_call_expression(&parsed.program.body[0]);
}

#[test]
#[should_panic]
fn test_first_call_expression_panics_when_not_call_expression() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "value;";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    first_call_expression(&parsed.program.body[0]);
}

#[test]
#[should_panic]
fn test_first_statement_assignment_call_expression_panics_when_not_expression_statement() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "if (true) {}";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    first_statement_assignment_call_expression(&parsed.program.body[0]);
}

#[test]
#[should_panic]
fn test_first_statement_assignment_call_expression_panics_when_not_assignment_expression() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "cache(() => {});";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    first_statement_assignment_call_expression(&parsed.program.body[0]);
}

#[test]
#[should_panic]
fn test_first_statement_assignment_call_expression_panics_when_right_not_call_expression() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "cachedFn = helper;";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    first_statement_assignment_call_expression(&parsed.program.body[0]);
}

#[test]
#[should_panic]
fn test_object_argument_from_call_expression_panics_when_not_object_argument() {
    let allocator = oxc_allocator::Allocator::default();
    let source = "fetch('/api', '/not-object')";
    let source_type = oxc_span::SourceType::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let call = first_call_expression(&parsed.program.body[0]);
    object_argument_from_call_expression(call);
}
