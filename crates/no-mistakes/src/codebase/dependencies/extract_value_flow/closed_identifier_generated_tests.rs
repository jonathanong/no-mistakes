use crate::codebase::dependencies::extract::{
    extract_import_facts_from_program_with_source_and_demands, ValueExpression,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn generated_return_facts_require_the_trusted_stripped_uuid_shape() {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("issue-3073-generated/src");
    for (file, count) in [
        ("temp-names.ts", 1),
        ("fake-uuid.ts", 0),
        ("negative-returns.ts", 2),
        ("writer.ts", 0),
    ] {
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{file}: {:?}",
            parsed.diagnostics
        );
        let facts = extract_import_facts_from_program_with_source_and_demands(
            &parsed.program,
            &source,
            false,
            false,
            true,
        );
        let generated = &facts.value_flow.generated_identifier_returns;
        assert_eq!(generated.len(), count, "{file}");
        for returned in generated {
            assert!(matches!(
                &returned.prefix,
                ValueExpression::Named { name, context: Some(owner), .. }
                    if name == "prefix" && owner == &returned.owner
            ));
        }
    }
}
