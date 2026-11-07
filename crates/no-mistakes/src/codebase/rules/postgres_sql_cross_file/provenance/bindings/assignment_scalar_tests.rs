use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

pub(super) fn prepared(names: &[&str]) -> (CheckFactMap, Vec<PathBuf>) {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("issue-3073-workspace-producer/apps/worker");
    let files: Vec<_> = names
        .iter()
        .map(|name| normalize_path(&root.join(name)))
        .collect();
    let plan = TsFactPlan {
        value_flow: true,
        ..TsFactPlan::imports()
    };
    let collected = collect_ts_facts(&files, plan);
    let mut facts = CheckFactMap {
        files: files.clone(),
        ..Default::default()
    };
    for file in &files {
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
    (facts, files)
}

pub(super) fn assigned<'a>(flow: &'a ValueFlowFacts, name: &str) -> &'a IdentifierAssignment {
    flow.identifier_writes
        .iter()
        .filter(|write| write.name == name)
        .find_map(|write| write.assigned.as_deref())
        .unwrap_or_else(|| panic!("missing assignment {name}"))
}

#[test]
fn scalar_assignment_forms_preserve_open_sources_and_finite_replacements() {
    let (facts, files) = prepared(&["assignment-open.tsx", "assignment-closed.tsx"]);
    let bindings = Bindings::new(&facts).with_numeric_parameter_bounds();
    for (index, file) in files.iter().enumerate() {
        let flow = &facts.get_ts_facts(file).unwrap().value_flow;
        for name in [
            "templateValue",
            "additionValue",
            "conditionalValue",
            "logicalValue",
            "awaitedValue",
            "staticValue",
            "computedValue",
            "sequenceValue",
            "conditionalWrite",
        ] {
            let binding = flow
                .bindings
                .iter()
                .find(|binding| binding.name == name)
                .unwrap();
            let write = assigned(flow, name);
            let call = flow
                .calls
                .iter()
                .filter(|call| {
                    call.context == binding.owner
                        && call.source_callee == "read"
                        && call.offset >= write.completed_at
                })
                .min_by_key(|call| call.offset)
                .unwrap();
            assert_eq!(
                bindings.flows_unbounded(file, binding.scope, name, call.end),
                index == 0,
                "{file:?} {name}"
            );
        }
        assert!(matches!(
            assigned(flow, "templateValue").value,
            ValueExpression::Derived(_)
        ));
        assert!(matches!(
            assigned(flow, "additionValue").value,
            ValueExpression::Derived(_)
        ));
        assert!(matches!(
            assigned(flow, "conditionalValue").value,
            ValueExpression::Either(_, _)
        ));
        assert!(matches!(
            assigned(flow, "logicalValue").value,
            ValueExpression::Either(_, _)
        ));
        assert!(matches!(
            assigned(flow, "awaitedValue").value,
            ValueExpression::Awaited(_)
        ));
        for name in ["staticValue", "computedValue"] {
            assert!(
                matches!(&assigned(flow, name).value, ValueExpression::Member(_, key) if key == "TABLE" || key == "table")
            );
        }
        assert!(matches!(
            assigned(flow, "sequenceValue").value,
            ValueExpression::Named { .. } | ValueExpression::Unknown
        ));
    }
}

#[path = "assignment_memory_tests.rs"]
mod memory;
