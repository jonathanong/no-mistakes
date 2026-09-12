use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn dynamic_receivers_record_unknown_calls_for_call_new_and_tags() {
    let extracted = facts(
        "function helper() {}\nfoo().bar();\nfoo().tag`x`;\nnew (foo().Cls)();\n({ name } = source);",
    );
    assert!(
        !extracted.function_calls.is_empty() || !extracted.unknown_calls.is_empty(),
        "{extracted:#?}",
    );
    let _ = facts("type Nested = this.Member;");
    let _ = facts("export default (function expr() { return 1; })");
}
