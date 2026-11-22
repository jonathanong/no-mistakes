use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn object_bindings_are_collected_only_when_scheduler_needs_a_lookup() {
    let allocator = Allocator::default();
    let binding_allocator = Allocator::default();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/queue-ast-hop");
    for (file, expected) in [
        ("invalid-job-id/fixture/configured-enqueue.ts", false),
        ("job-scheduler-template/fixture/disabled.ts", false),
        ("job-scheduler-template/fixture/orders.ts", true),
    ] {
        let path = fixture.join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty(), "{file}");
        let bindings = LazyObjectBindings::new(&parsed.program, &binding_allocator);
        let roles = Default::default();
        let visitor = extract_with_bindings(&path, &source, &[], &roles, false, None, bindings);
        assert_eq!(visitor.object_bindings.was_collected(), expected, "{file}");
    }
}

#[test]
fn object_binding_lookups_reuse_the_named_group() {
    let allocator = Allocator::default();
    let binding_allocator = Allocator::default();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/job-scheduler-template/fixture/arrow-scope.ts");
    let source = std::fs::read_to_string(path).unwrap();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let bindings = LazyObjectBindings::new(&parsed.program, &binding_allocator);
    assert!(!bindings.was_collected());

    let options = bindings.named("options");
    assert_eq!(options.len(), 4);
    assert_eq!(bindings.named("wrap").len(), 2);
    assert!(bindings.named("missing").is_empty());
    let option = bindings.lookup("options", 0).unwrap();
    assert!(std::ptr::eq(option, &options[0]));
    assert!(std::ptr::eq(options, bindings.named("options")));
}
