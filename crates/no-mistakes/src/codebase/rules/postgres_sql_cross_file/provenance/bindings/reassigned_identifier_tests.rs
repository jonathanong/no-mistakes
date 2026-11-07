use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::sync::Arc;

#[test]
fn reassigned_identifier_locals_lose_bounded_parameter_and_literal_initializers() {
    let unchecked = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-workspace-producer/apps/worker/unchecked.ts"),
    );
    let guarded = unchecked.with_file_name("guarded.ts");
    let files = [unchecked.clone(), guarded.clone()];
    let plan = TsFactPlan {
        value_flow: true,
        ..TsFactPlan::imports()
    };
    let collected = collect_ts_facts(&files, plan);
    let mut facts = CheckFactMap::default();
    facts.files.extend(files);
    for file in &facts.files {
        let ts = collected.get(file).unwrap();
        assert!(ts.parse_error.is_none(), "{:?}", ts.parse_error);
        facts.ts.insert(
            file.clone(),
            CheckFileFacts {
                ts: Arc::new(ts.clone()),
                ..Default::default()
            }
            .into(),
        );
    }
    let shared = Bindings::new(&facts);
    let identifier = Bindings::new(&facts).with_numeric_parameter_bounds();
    assert!(shared.assignments.get().is_none());
    assert!(identifier.assignments.get().is_none());
    let mut cached_index = None;
    for (file, name, unbounded) in [
        (&unchecked, "alias", true),
        (&unchecked, "literal", true),
        (&guarded, "finite", false),
        (&guarded, "counter", false),
    ] {
        let ts = collected.get(file).unwrap();
        let write = ts
            .value_flow
            .identifier_writes
            .iter()
            .find(|write| write.name == name && write.assigned.is_some())
            .unwrap();
        let assignment = write.assigned.as_deref().unwrap();
        let call = ts
            .value_flow
            .calls
            .iter()
            .find(|call| {
                call.context == assignment.owner
                    && call.source_callee == "read"
                    && call.offset >= assignment.completed_at
            })
            .unwrap();
        assert!(shared.value(file, write.scope, name, call.end).is_none());
        assert!(identifier
            .value(file, write.scope, name, call.end)
            .is_none());
        assert!(!shared.flows_unbounded(file, write.scope, name, call.end));
        assert!(shared.assignments.get().is_none());
        assert_eq!(
            identifier.flows_unbounded(file, write.scope, name, call.end),
            unbounded,
            "{name}"
        );
        let index = identifier.assignments.get().unwrap();
        if let Some(previous) = cached_index {
            assert!(std::ptr::eq(previous, index));
        }
        cached_index = Some(index);
        if name == "literal" {
            assert!(!identifier.flows_unbounded(file, write.scope, name, write.at - 1));
        }
    }
}
