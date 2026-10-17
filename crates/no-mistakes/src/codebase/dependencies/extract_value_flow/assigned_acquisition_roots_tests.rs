use super::*;
use std::cell::Cell;
thread_local! {
    pub(in super::super) static OLD: Cell<bool> = const { Cell::new(false) };
    pub(super) static ENTRIES: Cell<usize> = const { Cell::new(0) };
}
impl ImportCollector {
    pub(in super::super) fn record_rhs_acquisition_legacy(&mut self, value: &ValueExpression) {
        ENTRIES.with(|n| n.set(n.get() + 1));
        if self.receiver_only_bindings.is_none() {
            return;
        }
        match value {
            ValueExpression::Named { name, scope, .. } if !name.contains('.') => {
                if self.references_mutable_receiver(value) {
                    self.prior_member_mutations
                        .acquire_rhs_reference(scope.unwrap_or(0), name);
                }
            }
            ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
                for value in fields.values() {
                    self.record_rhs_acquisition_legacy(value);
                }
            }
            _ => {
                // Reuse the canonical descriptor projection, including static
                // member selectors, instead of scanning bindings or the AST.
                let mut references = Vec::new();
                nested::receiver_references(value, &[], &mut references);
                for ((scope, name), _) in references {
                    if self.has_mutable_receiver_binding(scope, &name) {
                        self.prior_member_mutations
                            .acquire_rhs_reference(scope, &name);
                    }
                }
            }
        }
    }
}

#[test]
fn acquisition_fullfacts_and_const_admission() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases");
    let nested = "rules/query-reached-per-item/provenance-fixture/src/nested-receiver-containers";
    let mut positive = false;
    for file in [
        "codebase-analysis/import-facts/fixture/many-late-defaults.mts".to_string(),
        format!("{nested}/assignment-rhs-member-source.mts"),
        "codebase-analysis/receiver-acquisition/assignment-rhs-borrowed-roots.mts".to_string(),
        format!("{nested}/assignment-rhs-pattern-default.mts"),
        format!("{nested}/assignment-rhs-copied.mts"),
        format!("{nested}/assignment-rhs-mutable-rebound.mts"),
        format!("{nested}/assignment-rhs-saved-choice.mts"),
    ] {
        let source = std::fs::read_to_string(root.join(&file)).unwrap();
        let allocator = Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty());
        for demand in [false, true] {
            let run = |old| {
                OLD.with(|v| v.set(old));
                ENTRIES.with(|v| v.set(0));
                let facts = crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands(
                    &parsed.program, &source, false, false, demand);
                OLD.with(|v| v.set(false));
                (facts, ENTRIES.with(Cell::get))
            };
            let (mut old, old_entries) = run(true);
            let (new, entries) = run(false);
            // This cfg-only counter intentionally measures the duplicate lookup
            // removed by bool reuse. Every other facts field must stay equal.
            let old_probes = old.value_flow.receiver_only_membership_probes;
            let new_probes = new.value_flow.receiver_only_membership_probes;
            assert!(old_probes >= new_probes, "{file}: demand={demand}");
            if file.ends_with("assignment-rhs-borrowed-roots.mts") && demand {
                assert_eq!(old_probes - new_probes, 1);
            }
            println!("MEMBERSHIP {file} demand={demand} old={old_probes} new={new_probes}");
            old.value_flow.receiver_only_membership_probes = new_probes;
            assert_eq!(old, new, "{file}: demand={demand}");
            if file.contains("many-late-defaults") && demand {
                assert!(old_entries > 0);
                assert_eq!(entries, 0);
                assert!(!new.value_flow.bindings.is_empty());
            }
            positive |= entries > 0;
            println!("ACQUISITION {file} demand={demand} old={old_entries} new={entries}");
        }
    }
    assert!(
        positive,
        "mutable catalog fixture must exercise acquisition"
    );
}

#[test]
fn borrowed_projection_matches_canonical_roots() {
    use std::sync::Arc;
    let named = |name: &str| ValueExpression::Named {
        name: name.to_string(),
        scope: None,
        offset: 0,
        context: None,
    };
    let mut fields = fx_map();
    fields.insert("picked".to_string(), named("selected.deep"));
    fields.insert("ignored".to_string(), named("unselected"));
    let object = ValueExpression::Object(Arc::new(fields.clone()), false);
    let array = ValueExpression::Array(Arc::new(fields));
    let projected = ValueExpression::Member(Arc::new(object.clone()), "picked".to_string());
    let nested = ValueExpression::Member(
        Arc::new(ValueExpression::Member(
            Arc::new(named("root.branch")),
            "inner".to_string(),
        )),
        "outer".to_string(),
    );
    for value in [
        named(""),
        named("plain"),
        named("a.b"),
        object,
        array.clone(),
        projected.clone(),
        nested,
        ValueExpression::Member(Arc::new(array), "missing".to_string()),
        ValueExpression::Awaited(Arc::new(projected.clone())),
        ValueExpression::Default(Arc::new(projected), Arc::new(named("fallback"))),
        ValueExpression::Unknown,
    ] {
        let mut canonical = Vec::new();
        nested::receiver_references(&value, &[], &mut canonical);
        let canonical: Vec<_> = canonical.into_iter().map(|(key, _)| key).collect();
        let mut borrowed = Vec::new();
        roots(&value, None, &mut |scope, name| {
            borrowed.push((scope, name.to_string()))
        });
        assert_eq!(canonical, borrowed, "{value:?}");
    }
}
