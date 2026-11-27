use super::*;

struct OldEntry(bool);
impl OldEntry {
    fn set(value: bool) -> Self {
        Self(OLD_ENTRY.with(|old| old.replace(value)))
    }
}
impl Drop for OldEntry {
    fn drop(&mut self) {
        OLD_ENTRY.with(|old| old.set(self.0));
    }
}

fn collect(source: &str, demand: bool, old: bool) -> (ImportFacts, usize) {
    let _mode = OldEntry::set(old);
    MEMBER_CLONES.with(|count| count.set(0));
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        false,
        false,
        demand,
    );
    (facts, MEMBER_CLONES.with(std::cell::Cell::get))
}

#[test]
fn owned_first_path_saved_full_facts_match_old_entry() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture");
    for fixture in [
        "mutable-root-scale.mts",
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
        "mutable-projection-wide.ts",
        "mutable-pattern-deferred-diamond.mts",
        "mutable-let-cycle.mts",
        "mutable-let-pattern-static-fields.mts",
        "guarded-dynamic.mts",
    ] {
        let source =
            std::fs::read_to_string(root.join("src/nested-receiver-containers").join(fixture))
                .unwrap();
        for demand in [false, true] {
            let (old, clones) = collect(&source, demand, true);
            let (new, moved) = collect(&source, demand, false);
            assert_eq!(old, new, "{fixture} demand={demand}");
            eprintln!("ACTUAL_MEMBER_CLONES {fixture} {demand} old={clones} new={moved}");
        }
    }
}

#[test]
fn owned_first_path_eliminates_saved_root_member_clones() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture");
    let source =
        std::fs::read_to_string(root.join("src/nested-receiver-containers/mutable-root-scale.mts"))
            .unwrap();
    let (old, clones) = collect(&source, true, true);
    assert_eq!(clones, 128);
    let old_mode = std::env::var_os("OWNED_FIRST_EXPECT_RED").is_some();
    let (new, moved) = collect(&source, true, old_mode);
    assert_eq!(old, new);
    assert_eq!(new.value_flow.receiver_only_visits, 128);
    eprintln!("ACTUAL_OWNED_FIRST_MEMBER_CLONES old={clones} selected={moved}");
    assert_eq!(moved, 0, "owned first path must not clone its member");
}
