use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn object_bindings_share_the_processor_pass_and_index_only_on_lookup() {
    let allocator = Allocator::default();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/queue-ast-hop");
    for (file, visits, indexed) in [
        ("invalid-job-id/fixture/configured-enqueue.ts", 1, true),
        ("job-scheduler-template/fixture/plain.ts", 1, false),
        ("job-scheduler-template/fixture/orders.ts", 1, true),
    ] {
        let path = fixture.join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty(), "{file}");
        let roles = Default::default();
        let visitor =
            extract_queue_facts(&path, &source, &parsed.program, &[], &roles, false, None);
        assert_eq!(visitor.object_bindings.program_visits, visits, "{file}");
        assert_eq!(visitor.object_bindings.was_indexed(), indexed, "{file}");
    }
}

#[test]
fn object_binding_lookups_reuse_the_named_group() {
    let allocator = Allocator::default();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/job-scheduler-template/fixture/arrow-scope.ts");
    let source = std::fs::read_to_string(path).unwrap();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let index =
        super::super::extract_factory::queue_index(&parsed.program, &Default::default(), true);
    let bindings = index.object_bindings;
    assert!(!bindings.was_indexed());
    assert_eq!(bindings.program_visits, 1);

    let options = bindings.named("options");
    assert_eq!(options.len(), 4);
    assert_eq!(bindings.named("wrap").len(), 2);
    assert!(bindings.named("missing").is_empty());
    let option = bindings.lookup("options", 0).unwrap();
    assert!(std::ptr::eq(option, &options[0]));
    assert!(std::ptr::eq(options, bindings.named("options")));
}

#[test]
fn forward_scheduler_bindings_and_worker_processors_are_prepared_together() {
    let allocator = Allocator::default();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/scheduler-binding-pass/fixture/forward.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let roles = Default::default();
    let visitor = extract_queue_facts(&path, &source, &parsed.program, &[], &roles, false, None);
    assert_eq!(visitor.facts.workers.len(), 1);
    assert_eq!(visitor.facts.workers[0].jobs, ["pack"]);
    assert_eq!(visitor.object_bindings.program_visits, 1);
    assert_eq!(
        visitor
            .facts
            .checks
            .iter()
            .filter(|check| check.kind == "scheduler-template-option")
            .map(|check| check.line)
            .collect::<Vec<_>>(),
        [14, 15]
    );
}
