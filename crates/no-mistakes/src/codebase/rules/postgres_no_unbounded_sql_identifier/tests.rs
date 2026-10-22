use super::{export_is_unbounded, identifier_finding, RULE_ID};
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    ExportedBinding, ImportedBinding, ImportedBindingKind,
};
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::rules::postgres_sql_cross_file::{Provenance, SqlCall};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn root(scenario: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture")
        .join(scenario)
}

fn findings(scenario: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    let fixture = root(scenario);
    let config = crate::config::v2::load_v2_config(&fixture, None).unwrap();
    // A passing control must exercise an enabled application, not an empty run.
    assert!(
        config.rule_configured(RULE_ID),
        "inactive fixture: {scenario}"
    );
    crate::codebase::rules::run_check(&fixture, None, None)
        .unwrap()
        .into_iter()
        .filter(|finding| finding.rule == RULE_ID)
        .collect()
}

#[test]
fn env_imported_config_and_type_assertions_are_unbounded() {
    for scenario in [
        "fail-env",
        "fail-imported-config",
        "fail-type-assertion",
        "fail-identity-wrapper",
        "fail-to-lowercase",
        "fail-bracket-env",
        "fail-imported-env",
        "fail-imported-process",
        "fail-request-params",
        "fail-db-rows",
    ] {
        let found = findings(scenario);
        assert!(
            found
                .iter()
                .any(|finding| finding.file.ends_with("query.ts")
                    && finding.message.contains("finite-domain validator")),
            "{scenario}: {found:#?}"
        );
    }
}

#[test]
fn switch_bind_http_and_suppression_stay_silent() {
    for scenario in [
        "pass-switch",
        "pass-bind-value",
        "pass-http-query",
        "pass-suppressed",
        "pass-imported-literal",
    ] {
        assert!(
            findings(scenario).is_empty(),
            "{scenario}: {:#?}",
            findings(scenario)
        );
    }
}

#[test]
fn imported_config_without_prepared_producer_facts_is_not_unbounded() {
    assert!(!export_is_unbounded(
        &CheckFactMap::default(),
        Path::new("/repo/src/config.ts"),
        "config"
    ));
}

#[test]
fn same_file_or_literal_exports_are_not_unbounded_identifier_sources() {
    let file = PathBuf::from("/repo/src/query.ts");
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                imported_bindings: vec![ImportedBinding {
                    specifier: "./query".into(),
                    local: "schema".into(),
                    imported: "schema".into(),
                    kind: ImportedBindingKind::Named,
                    is_type_only: false,
                }],
                exported_bindings: vec![ExportedBinding {
                    specifier: None,
                    local: "schema".into(),
                    exported: "schema".into(),
                }],
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
    );
    let mut forward = HashMap::new();
    forward.insert(file.clone(), vec![file.clone()]);
    let graph = from_raw_maps(PathBuf::from("/repo"), forward, Default::default());
    let owned = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: file.clone(),
        span: (0, 1),
        call: &owned,
    };
    assert!(identifier_finding(
        Path::new("/repo"),
        &graph,
        &facts,
        &super::RegistryRoutes::new(&facts),
        &call,
        &Provenance::Reference {
            name: "schema".into(),
            scope: 0,
            offset: 0,
            members: vec![]
        },
    )
    .is_none());
}

#[test]
fn shadowed_process_and_unbounded_namespace_members_are_classified() {
    let file = PathBuf::from("/repo/src/query.ts");
    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                imported_bindings: vec![ImportedBinding {
                    specifier: "./process".into(),
                    local: "process".into(),
                    imported: "process".into(),
                    kind: ImportedBindingKind::Named,
                    is_type_only: false,
                }],
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
    let owned = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: file.clone(),
        span: (0, 1),
        call: &owned,
    };
    assert!(identifier_finding(
        Path::new("/repo"),
        &graph,
        &facts,
        &super::RegistryRoutes::new(&facts),
        &call,
        &Provenance::ProcessEnv("SCHEMA".into()),
    )
    .is_none());
    assert!(identifier_finding(
        Path::new("/repo"),
        &graph,
        &facts,
        &super::RegistryRoutes::new(&facts),
        &call,
        &Provenance::Namespace {
            local: "t".into(),
            exported: "config".into(),
            members: vec!["config".into()],
        },
    )
    .is_none());
}

#[path = "recall_shapes_tests.rs"]
mod recall_shapes;
