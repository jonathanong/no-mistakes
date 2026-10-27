use super::write_index::WriteIndex;
use super::writes::mutated_exports;

// Output cases moved to cli_postgres_snapshot_no_mutable_sql_registry.
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{ValueCall, ValueExpression, ValueFlowFacts};
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn member_writes_abstain_without_facts_or_when_the_writer_did_not_parse() {
    let writer = PathBuf::from("/repo/src/mutate.ts");
    let facts = CheckFactMap::default();
    assert!(!WriteIndex::new(facts.get_ts_facts(&writer)).member_write("tables"));
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
    assert!(mutated_exports(&graph, &facts, &super::RegistryRoutes::new(&facts)).is_empty());
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
        assert!(
            WriteIndex::new(facts.get_ts_facts(&writer)).member_write("tables"),
            "{base}.{method}"
        );
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
    assert!(!WriteIndex::new(facts.get_ts_facts(&writer)).member_write("tables"));
}
