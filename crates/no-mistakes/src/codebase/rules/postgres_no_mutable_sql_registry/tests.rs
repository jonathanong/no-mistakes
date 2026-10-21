use super::writes::{member_write, mutated_exports};
use super::RULE_ID;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{ValueCall, ValueExpression, ValueFlowFacts};
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

fn root(scenario: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-mutable-sql-registry/fixture")
        .join(scenario)
}

fn findings(scenario: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    crate::codebase::rules::run_check(&root(scenario), None, None)
        .unwrap()
        .into_iter()
        .filter(|finding| finding.rule == RULE_ID)
        .collect()
}

#[test]
fn member_write_alias_and_array_assignment_reach_sql() {
    for scenario in [
        "fail-member-write",
        "fail-computed-key",
        "fail-alias",
        "fail-array",
        "fail-namespace",
        "fail-reexport-alias",
        "fail-export-star",
        "fail-delete",
        "fail-define-property",
    ] {
        let found = findings(scenario);
        assert!(
            found.iter().any(|finding| {
                finding.file.ends_with("query.ts")
                    && finding.message.contains("src/tables.ts")
                    && finding.message.contains("src/mutate.ts")
            }),
            "{scenario}: {found:#?}"
        );
    }
}

#[test]
fn literal_unrelated_prototype_bind_http_and_suppression_stay_silent() {
    for scenario in [
        "pass-literal-registry",
        "pass-unrelated-state",
        "pass-bind-value",
        "pass-http-query",
        "pass-suppressed",
        "pass-env-identifier",
        "pass-same-file-write",
    ] {
        assert!(
            findings(scenario).is_empty(),
            "{scenario}: {:#?}",
            findings(scenario)
        );
    }
}

#[test]
fn member_writes_abstain_without_facts_or_when_the_writer_did_not_parse() {
    let writer = PathBuf::from("/repo/src/mutate.ts");
    let facts = CheckFactMap::default();
    assert!(!member_write(&facts, &writer, "tables"));
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        graph_files: vec![writer.clone()],
        ..Default::default()
    };
    facts.ts.insert(
        writer,
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                parse_error: Some("boom".into()),
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
    );
    let graph = from_raw_maps(
        PathBuf::from("/repo"),
        Default::default(),
        Default::default(),
    );
    assert!(mutated_exports(&graph, &facts).is_empty());
}

fn named(name: &str) -> ValueExpression {
    ValueExpression::Named {
        name: name.into(),
        scope: None,
        offset: 0,
        context: None,
    }
}

fn mutator_call(base: &str, method: &str) -> ValueCall {
    ValueCall {
        offset: 0,
        end: 1,
        source_callee: format!("{base}.{method}"),
        context: None,
        callee: ValueExpression::Member(Arc::new(named(base)), method.into()),
        arguments: Arc::new(vec![named("tables")]),
        bound: false,
        spread_at: None,
        once: false,
        repeated: false,
    }
}

#[test]
fn object_and_reflect_property_mutators_are_member_writes() {
    let writer = PathBuf::from("/repo/src/mutate.ts");
    for (base, method) in [
        ("Object", "setPrototypeOf"),
        ("Reflect", "defineProperty"),
        ("Reflect", "deleteProperty"),
    ] {
        let mut facts = CheckFactMap::default();
        facts.ts.insert(
            writer.clone(),
            CheckFileFacts {
                ts: Arc::new(TsFileFacts {
                    value_flow: ValueFlowFacts {
                        calls: vec![mutator_call(base, method)],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                ..Default::default()
            }
            .into(),
        );
        assert!(member_write(&facts, &writer, "tables"), "{base}.{method}");
    }
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        writer.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                value_flow: ValueFlowFacts {
                    calls: vec![ValueCall {
                        callee: named("assign"),
                        arguments: Arc::new(vec![named("tables")]),
                        source_callee: "assign".into(),
                        offset: 0,
                        end: 1,
                        context: None,
                        bound: false,
                        spread_at: None,
                        once: false,
                        repeated: false,
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
    );
    assert!(!member_write(&facts, &writer, "tables"));
}

#[path = "recall_shapes_tests.rs"]
mod recall_shapes;
