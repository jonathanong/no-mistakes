use super::super::*;
use oxc_allocator::Allocator;
use oxc_span::SourceType;

#[test]
fn optional_push_tail_work_is_linear_and_keeps_the_late_origin() {
    for (name, emitted) in [("optional-tail-stress", 97), ("optional-tail-control", 0)] {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/value-flow-guarded-array-replay/fixture/src")
            .join(format!("{name}.mts"));
        let source = std::fs::read_to_string(path).unwrap();
        let allocator = Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty());
        for enabled in [false, true] {
            super::WORK.with(|work| work.set([0; 2]));
            let facts = extract_import_facts_from_program_with_source_and_demands(
                &parsed.program,
                &source,
                false,
                false,
                enabled,
            );
            let work = super::WORK.with(std::cell::Cell::get);
            if !enabled {
                assert_eq!(work, [0; 2]);
                continue;
            }
            // One fixed prefix plus 97 authored pushes; tail values are emitted
            // once each, never once for every possible incoming array length.
            assert_eq!(work, [98, emitted], "{name}");
            if emitted != 0 {
                let possible = facts
                    .value_flow
                    .possible_receiver_mutations
                    .as_ref()
                    .unwrap();
                for origin in ["replica", "untouched"] {
                    assert!(
                        possible[&0][origin].assignment_invalidated(
                            &["get"],
                            u32::MAX,
                            None,
                            false
                        ),
                        "{name}: missing {origin}"
                    );
                }
            }
        }
    }
}
