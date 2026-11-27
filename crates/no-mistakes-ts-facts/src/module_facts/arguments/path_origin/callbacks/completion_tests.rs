use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn synchronous_callback_completion_follows_only_bounded_blocks_and_sync_loops() {
    let path = crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts")
        .join("callback-completion.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::from_path(path).unwrap()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let actual: Vec<_> = parsed
        .program
        .body
        .iter()
        .map(|statement| {
            let Statement::FunctionDeclaration(function) = statement else {
                panic!("expected function");
            };
            (
                function.id.as_ref().unwrap().name.as_str(),
                synchronous_body(function.body.as_ref().unwrap()),
            )
        })
        .collect();
    assert_eq!(
        actual,
        [
            ("blocks", true),
            ("loop", true),
            ("directLoop", true),
            ("returnedPromise", false),
            ("loopReturnedPromise", false),
            ("awaitedLoop", false),
            ("depthAccepted", true),
            ("depthRejected", false),
        ]
    );
}
