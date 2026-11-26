use super::*;

fn saved_source(relative: &str) -> (std::path::PathBuf, String) {
    // Reuse the per-crate saved-fixture root; these cases span both fixture trees.
    let fixture_root = crate::test_support::rule_fixture_root("../..");
    let path = fixture_root.parent().unwrap().join(relative);
    let source = std::fs::read_to_string(&path).unwrap();
    (path, source)
}

#[test]
fn statement_outline_preserves_whole_import_facts_for_every_demand() {
    let mut pairs = 0;
    for relative in [
        "fixtures/performance/react-traits/many-components.tsx",
        "test-cases/codebase-analysis/import-facts/fixture/replacement-demand.mts",
        "test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts",
        "test-cases/codebase-analysis/value-flow-foreign-parameter-timing/fixture/src/entry.mts",
        "test-cases/codebase-analysis/value-flow-foreign-parameter-timing/fixture/src/helper.mts",
        "test-cases/codebase-analysis/value-flow-foreign-parameter-timing/fixture/src/writer.mts",
        "test-cases/codebase-analysis/value-flow-replacement-cold/fixture/src/entry.mts",
        "test-cases/codebase-analysis/inline-member-precision/fixture/src/loop-setup-annex.cjs",
    ] {
        let (path, source) = saved_source(relative);
        let allocator = Allocator::default();
        let parsed =
            Parser::new(&allocator, &source, SourceType::from_path(&path).unwrap()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{relative}: {:?}",
            parsed.diagnostics
        );
        for values in [false, true] {
            for resources in [false, true] {
                for per_item in [false, true] {
                    let collect = |original| {
                        statement_mode::with_original_statements(original, || {
                            extract_import_facts_from_program_with_source_and_demands(
                                &parsed.program,
                                &source,
                                resources,
                                per_item,
                                values,
                            )
                        })
                    };
                    let old = collect(true);
                    let new = collect(false);
                    // Whole facts include every scope, import, call, origin and cfg work field.
                    assert_eq!(
                        old, new,
                        "{relative}: values={values}, resources={resources}, per_item={per_item}"
                    );
                    let entries = new.value_flow.replacement_statement_checks;
                    if values {
                        assert!(entries > 0);
                    } else {
                        assert_eq!(entries, 0);
                    }
                    println!(
                        "STATEMENT_OUTLINE_ORACLE {}",
                        serde_json::json!({"fixture":relative,"values":values,"resources":resources,"per_item":per_item,"old_checks":old.value_flow.replacement_statement_checks,"new_checks":entries,"complete_facts_equal":true})
                    );
                    pairs += 1;
                }
            }
        }
    }
    assert_eq!(pairs, 64);
}
