use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::sync::Arc;

#[test]
fn assignment_provenance_keeps_numeric_bounds_and_conservative_parameter_origins() {
    let file = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("3304-registry-routes/src/assignments.ts"),
    );
    let source = std::fs::read_to_string(&file).unwrap();
    let files = [file.clone()];
    let plan = TsFactPlan {
        value_flow: true,
        identifier_provenance: true,
        ..TsFactPlan::imports()
    };
    let ts = collect_ts_facts(&files, plan).remove(&file).unwrap();
    assert!(ts.parse_error.is_none(), "{:?}", ts.parse_error);
    let mut facts = CheckFactMap {
        files: files.to_vec(),
        ..Default::default()
    };
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
    let bindings = Bindings::new(&facts).with_numeric_parameter_bounds();
    let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
    let mut checked = 0;
    let mut mismatches = Vec::new();
    for call in flow
        .calls
        .iter()
        .filter(|call| call.source_callee == "probe")
    {
        let line = source[call.offset as usize..].lines().next().unwrap();
        let expected: bool = line.split_once("// open: ").unwrap().1.parse().unwrap();
        let [ValueExpression::Named { name, scope, .. }] = call.arguments.as_slice() else {
            panic!("probe must read a named binding: {line}")
        };
        let actual = bindings.flows_unbounded(&file, scope.unwrap_or(0), name, call.end);
        if actual != expected {
            mismatches.push(format!("{line}: got {actual}, expected {expected}"));
        }
        checked += 1;
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    assert_eq!(checked, 5);
}
