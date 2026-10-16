use super::*;
thread_local! {
    pub(in super::super) static LEGACY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(in super::super) static LEGACY_IDENTIFIER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
struct Mode(bool);
impl Drop for Mode {
    fn drop(&mut self) {
        LEGACY.with(|mode| mode.set(self.0));
    }
}
fn collect(source: &str, legacy: bool, demand: bool) -> ImportFacts {
    let _mode = Mode(LEGACY.with(|mode| mode.replace(legacy)));
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        false,
        false,
        demand,
    )
}
#[test]
fn shared_deferred_frame_preserves_all_facts() {
    for name in [
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
        "mutable-pattern-deferred-diamond.mts",
        "mutable-pattern-future-foreign-32.mts",
        "mutable-pattern-future-foreign-128.mts",
        "mutable-let-conditional-root.mts",
        "mutable-let-foreign-root.mts",
        "mutable-let-cycle.mts",
        "mutable-let-pattern-conditional-rebind.mts",
        "mutable-let-pattern-conditional-whole.mts",
        "mutable-let-pattern-class-context.mts",
        "mutable-let-pattern-shadowed-class.mts",
    ] {
        let source = mutable_receiver_source(name);
        for demand in [false, true] {
            let old = collect(&source, true, demand);
            let new = collect(&source, false, demand);
            // Frame counters are thread-local, so no semantic field is masked.
            assert_eq!(new, old, "{name}: demand={demand}");
        }
    }
}

#[test]
fn phase_names_allocate_only_for_distinct_keys() {
    root_target_metrics::PHASE_KEYS.with(|counts| counts.set([0; 4]));
    let source = mutable_receiver_source("mutable-pattern-phase-scale.mts");
    let _facts = collect(&source, false, true);
    let counts = root_target_metrics::PHASE_KEYS.with(std::cell::Cell::get);
    eprintln!("phase key attempts/owned allocations {counts:?}");
    assert_eq!(counts, [63, 32, 32, 1]);
}

#[test]
fn phase_identifier_metadata_avoids_owned_targets() {
    let mut observed = Vec::new();
    for name in [
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
    ] {
        root_target_metrics::PHASE_IDENTIFIERS.with(|count| count.set(0));
        let source = mutable_receiver_source(name);
        let _facts = collect(&source, false, true);
        let count = root_target_metrics::PHASE_IDENTIFIERS.with(std::cell::Cell::get);
        eprintln!("{name}: phase owned identifier target constructions {count}");
        observed.push((name, count));
    }
    assert!(
        observed.iter().all(|(_, count)| *count == 0),
        "{observed:?}"
    );
}

struct IdentifierMode(bool);
impl Drop for IdentifierMode {
    fn drop(&mut self) {
        LEGACY_IDENTIFIER.with(|mode| mode.set(self.0));
    }
}
#[test]
fn borrowed_phase_identifier_preserves_all_facts() {
    for name in [
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
        "mutable-pattern-deferred-diamond.mts",
        "mutable-pattern-future-foreign-32.mts",
        "mutable-let-pattern-conditional-rebind.mts",
        "mutable-let-pattern-conditional-whole.mts",
        "mutable-let-pattern-class-context.mts",
        "mutable-let-pattern-shadowed-class.mts",
        "mutable-let-pattern-static-capture.mts",
        "mutable-let-pattern-static-fields.mts",
        "mutable-let-pattern-wrapped.mts",
        "mutable-let-pattern-shorthand.mts",
        "mutable-root-target-metadata.ts",
    ] {
        let source = mutable_receiver_source(name);
        for demand in [false, true] {
            let old = {
                let _mode = IdentifierMode(LEGACY_IDENTIFIER.with(|mode| mode.replace(true)));
                collect(&source, false, demand)
            };
            let new = collect(&source, false, demand);
            assert_eq!(new, old, "{name}: demand={demand}");
        }
    }
}
