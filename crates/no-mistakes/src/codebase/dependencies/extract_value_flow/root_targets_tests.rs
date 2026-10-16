use super::*;
use oxc_ast_visit::Visit;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/provenance-fixture/src/nested-receiver-containers")
        .join(name)
}

struct CompareRoots;
impl<'a> Visit<'a> for CompareRoots {
    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        let expected: Vec<_> = write_targets::targets(&assignment.left)
            .into_iter()
            .filter(|(_, path, _)| path.is_empty())
            .map(|(name, _, start)| (name, start))
            .collect();
        let mut actual = Vec::new();
        for_each_root(&assignment.left, |name, start| {
            actual.push((name.to_owned(), start))
        });
        assert_eq!(actual, expected);
        oxc_ast_visit::walk::walk_assignment_expression(self, assignment);
    }
}
#[test]
fn borrowed_roots_preserve_order_and_physical_offsets() {
    for name in [
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
        "mutable-root-target-metadata.ts",
    ] {
        let file = fixture(name);
        let source = std::fs::read_to_string(&file).unwrap();
        let allocator = Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &source, SourceType::from_path(&file).unwrap())
                .parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{name}: {:?}",
            parsed.diagnostics
        );
        CompareRoots.visit_program(&parsed.program);
    }
}
#[cfg(feature = "test-instrumentation")]
struct ScanPatterns;
#[cfg(feature = "test-instrumentation")]
impl<'a> Visit<'a> for ScanPatterns {
    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if matches!(
            assignment.left,
            AssignmentTarget::ArrayAssignmentTarget(_)
                | AssignmentTarget::ObjectAssignmentTarget(_)
        ) {
            // Root preparation and root recording both need this same metadata.
            for _ in 0..2 {
                for_each_root(&assignment.left, |_, _| {});
            }
        }
        oxc_ast_visit::walk::walk_assignment_expression(self, assignment);
    }
}
#[test]
#[cfg(feature = "test-instrumentation")]
fn root_metadata_skips_full_target_constructors() {
    let mut observed = Vec::new();
    for name in [
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
    ] {
        let file = fixture(name);
        let source = std::fs::read_to_string(&file).unwrap();
        let allocator = Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &source, SourceType::from_path(&file).unwrap())
                .parse();
        assert!(parsed.diagnostics.is_empty());
        root_target_metrics::CONSTRUCTIONS.with(|counts| counts.set([0; 2]));
        ScanPatterns.visit_program(&parsed.program);
        observed.push((
            name,
            root_target_metrics::CONSTRUCTIONS.with(|counts| counts.get()),
        ));
    }
    println!("ROOT_METADATA_CONSTRUCTIONS {observed:?}");
    assert!(observed.iter().all(|(_, counts)| *counts == [0; 2]));
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn rhs_root_capture_does_not_construct_discarded_member_writes() {
    let mut discarded = Vec::new();
    for name in [
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
    ] {
        let file = fixture(name);
        let source = std::fs::read_to_string(&file).unwrap();
        let allocator = Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &source, SourceType::from_path(&file).unwrap())
                .parse();
        assert!(parsed.diagnostics.is_empty());
        root_target_metrics::RHS_DISCARDED_WRITES.with(|count| count.set(0));
        let facts = extract_import_facts_from_program_with_source_and_demands(
            &parsed.program,
            &source,
            false,
            false,
            true,
        );
        assert!(facts.value_flow.receiver_mutations.is_some());
        discarded.push((
            name,
            root_target_metrics::RHS_DISCARDED_WRITES.with(std::cell::Cell::get),
        ));
    }
    println!("RHS_DISCARDED_WRITES {discarded:?}");
    assert!(discarded.iter().all(|(_, count)| *count == 0));
}
