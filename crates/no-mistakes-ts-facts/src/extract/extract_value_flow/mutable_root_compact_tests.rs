use super::*;

fn facts(source: &str, disabled: bool) -> ImportFacts {
    mutable_root_single::DISABLED.with(|flag| flag.set(disabled));
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        true,
        true,
        true,
    );
    mutable_root_single::DISABLED.with(|flag| flag.set(false));
    facts
}
#[test]
fn compact_and_full_objects_have_identical_complete_facts() {
    for name in [
        "mutable-root-scale.mts",
        "mutable-root-single-fields.mts",
        "mutable-root-single-fallbacks.mts",
        "mutable-let-pattern-lhs.mts",
        "mutable-let-pattern-static-rebind.mts",
        "mutable-let-conditional-root.mts",
        "mutable-let-foreign-root.mts",
        "mutable-let-shadowed.mts",
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
    ] {
        let source = mutable_receiver_source(name);
        assert_eq!(facts(&source, false), facts(&source, true), "{name}");
    }
}
#[test]
fn repeated_names_have_one_lazy_bounded_interner() {
    mutable_origins::single::COLD.with(|cold| cold.set([0; 5]));
    flow(&mutable_receiver_source("mutable-root-scale.mts"));
    let cold = mutable_origins::single::COLD.with(std::cell::Cell::get);
    println!(
        "COMPACT_COLD tables={} distinct={} map_capacity={} vec_capacity={} payload_bytes={}",
        cold[0], cold[1], cold[2], cold[3], cold[4]
    );
    assert_eq!(cold[0], 1);
    assert_eq!(cold[1], 1);
    assert_eq!(cold[4], "client".len());
    mutable_origins::single::COLD.with(|cold| cold.set([0; 5]));
    flow(&mutable_receiver_source("mutable-let-shadowed.mts"));
    assert_eq!(
        mutable_origins::single::COLD.with(std::cell::Cell::get)[0],
        0
    );
}

#[test]
fn missing_root_catalog_keeps_the_full_converter_fallback() {
    let source = mutable_receiver_source("mutable-root-single-fields.mts");
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let assignment = parsed
        .program
        .body
        .iter()
        .find_map(|statement| {
            if let Statement::ExpressionStatement(statement) = statement {
                if let Expression::AssignmentExpression(assignment) = &statement.expression {
                    return Some(assignment);
                }
            }
            None
        })
        .unwrap();
    let mut collector = ImportCollector::default();
    assert!(!collector.record_single_named_root(0, "box", assignment));
    assert!(collector.receiver_only_bindings.is_none());
}
