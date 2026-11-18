use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::sync::Arc;

#[test]
fn saved_finite_tuple_annotations_and_some_guards_preserve_contracts() {
    let file =
        crate::test_support::fixture_root("closed-sql-identifier-domains").join("indexed.ts");
    let plan = TsFactPlan {
        value_flow: true,
        ..TsFactPlan::imports()
    };
    let ts = collect_ts_facts(std::slice::from_ref(&file), plan)
        .remove(&file)
        .unwrap();
    assert!(ts.parse_error.is_none());
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
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
    for (name, expected) in [
        ("finite", true),
        ("direct", true),
        ("alias", true),
        ("wrapped", true),
        ("checked", true),
        ("parenthesized", true),
        ("wide", false),
        ("open", false),
        ("shadowed", false),
    ] {
        let parameter = flow
            .parameters
            .iter()
            .find(|parameter| parameter.name == name)
            .unwrap();
        assert_eq!(bindings.closed_type(&file, parameter), expected, "{name}");
    }
    let guards = flow
        .finite_identifier_guards
        .iter()
        .filter(|guard| {
            matches!(
                guard.kind,
                crate::codebase::dependencies::extract::GuardKind::ClosedSet { .. }
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(guards.len(), 1);
    assert_eq!(guards[0].name, "guardedName");
}
