use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

fn tsx_facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn extract_walks_default_wrappers_assignment_aliases_and_syntax_edges() {
    let _ = facts(
        r#"
        export default foo!;
        export default <number>bar;
        export default (() => 1);
        export default (function wrapped() { return 1; });
        const helper = () => 1;
        const { fn = helper, ['computed']: skipped, missing } = { fn: helper, other: helper };
        const [first = helper, , third] = [helper, helper, helper];
        const { nested: { inner = helper } = { inner: helper } } = { nested: { inner: helper } };
        (helper)();
        helper[name]();
        class Box {
          accessor typed: number = helper();
          accessor empty;
          @deco
          accessor decorated = 1;
        }
        function deco(_target: unknown, _context: unknown) {}
        "#,
    );
    let _ = tsx_facts("export function Icon() { return <this.div />; }");
}
