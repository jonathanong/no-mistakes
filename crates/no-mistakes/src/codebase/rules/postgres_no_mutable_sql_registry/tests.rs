use super::{member_write, mutated_exports, RULE_ID};
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
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
    for scenario in ["fail-member-write", "fail-alias", "fail-array"] {
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
        "pass-prototype-key",
        "pass-bind-value",
        "pass-http-query",
        "pass-suppressed",
        "pass-namespace",
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
