use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn saved_source() -> String {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    std::fs::read_to_string(root.join("src/review-array-from.mts")).unwrap()
}

#[test]
fn array_from_only_propagates_known_zero_length() {
    let source = saved_source();
    let allocator = Allocator::default();
    let program = Parser::new(&allocator, &source, SourceType::mjs().with_typescript(true))
        .parse()
        .program;
    let facts = collect(&program);
    for (needle, expected) in [
        ("Array.from([]).forEach", Some(&[][..])),
        ("Array.from([,,]).forEach", Some(&[0][..])),
        ("Array.from([,,].slice(start)).forEach", Some(&[0][..])),
        ("Array.from(sparse).forEach", Some(&[0][..])),
    ] {
        let start = source.find(needle).unwrap() as u32;
        let match_for_call = facts
            .callbacks
            .iter()
            .find(|((offset, end), _)| {
                *offset == start && source[start as usize..*end as usize].contains(".forEach")
            })
            .map(|(_, positions)| *positions)
            .expect("fixture callback call has provenance metadata");
        assert_eq!(match_for_call, expected, "{needle}");
    }
}
