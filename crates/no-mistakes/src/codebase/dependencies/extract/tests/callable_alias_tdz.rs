use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn immutable_alias_records_its_declaration_offset() {
    let source = "function target() {} function run() { alias(); const alias = target; alias(); }";
    let facts = facts(source);
    let alias = facts
        .callable_aliases
        .iter()
        .find(|alias| alias.local == "alias")
        .expect("alias");
    let before = facts
        .function_calls
        .iter()
        .find(|call| call.callee == "alias")
        .expect("tdz call");
    let after = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "alias")
        .nth(1)
        .expect("live call");

    assert!(before.offset < alias.declared_at);
    assert!(after.offset >= alias.declared_at);
    assert_eq!(alias.target, "target");
}

#[test]
fn nested_closure_aliases_a_later_direct_arrow_binding() {
    let source = r#"
        function outer() {
          function nested() {
            const inner = later;
            inner();
          }
          const later = () => import("./target.mts");
          nested();
        }
    "#;
    let facts = facts(source);

    assert!(facts.callable_aliases.iter().any(|alias| {
        alias.local == "inner" && alias.target == "later" && alias.declared_at > 0
    }));
    assert!(facts
        .callable_binding_declared_at
        .iter()
        .any(|(_, name, _)| name == "later"));
}
