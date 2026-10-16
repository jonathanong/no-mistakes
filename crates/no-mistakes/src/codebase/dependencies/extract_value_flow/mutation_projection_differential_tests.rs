use super::*;
#[path = "mutation_projection_oracle_tests.rs"]
mod oracle;
thread_local! {
    pub(super) static LEGACY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
struct Mode(bool);
impl Drop for Mode {
    fn drop(&mut self) {
        LEGACY.with(|mode| mode.set(self.0));
    }
}
fn collect(source: &str, source_type: SourceType, legacy: bool, demand: bool) -> ImportFacts {
    let _mode = Mode(LEGACY.with(|mode| mode.replace(legacy)));
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        false,
        false,
        demand,
    )
}
#[test]
fn moved_keys_preserve_full_projection_facts_and_pending_capacity() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture");
    for name in [
        "mutable-root-scale.mts",
        "mutable-let-cycle.mts",
        "mutable-pattern-deferred-diamond.mts",
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
        "mutable-pattern-future-foreign-32.mts",
        "mutable-pattern-future-foreign-128.mts",
        "mutable-let-conditional-root.mts",
        "mutable-let-foreign-root.mts",
        "mutable-root-target-metadata.ts",
        "mutable-projection-wide.ts",
    ] {
        let file = root.join("src/nested-receiver-containers").join(name);
        let source = std::fs::read_to_string(&file).unwrap();
        for demand in [false, true] {
            let mut old = collect(&source, SourceType::from_path(&file).unwrap(), true, demand);
            let new = collect(
                &source,
                SourceType::from_path(&file).unwrap(),
                false,
                demand,
            );
            if demand && name == "mutable-projection-wide.ts" {
                println!(
                    "WIDE_PENDING old={} new={}",
                    old.value_flow.reference_pending_capacity,
                    new.value_flow.reference_pending_capacity
                );
                assert!(old.value_flow.reference_pending_capacity >= 32);
            }
            // Only cfg-only heap diagnostics change; every semantic field stays exact.
            assert!(
                new.value_flow.reference_pending_capacity
                    <= old.value_flow.reference_pending_capacity
            );
            assert!(
                new.value_flow.reference_pending_bytes <= old.value_flow.reference_pending_bytes
            );
            old.value_flow.reference_pending_capacity = new.value_flow.reference_pending_capacity;
            old.value_flow.reference_pending_bytes = new.value_flow.reference_pending_bytes;
            assert_eq!(new, old, "{name} demand={demand}");
        }
    }
}
