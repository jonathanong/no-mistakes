use super::project;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::query_annotation;
use crate::codebase::postgres::{extract_embedded_sql_from_program, EmbeddedSqlOptions};
use crate::codebase::ts_source::{
    facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
    FileIdMap,
};
use crate::diagnostics::{with_observer, InvocationObserver};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn prepared() -> FileIdMap<CheckFileFacts> {
    prepared_fixture(
        "helper-tracing-target-reuse",
        &[
            "entry",
            "barrel",
            "origin",
            "left",
            "right",
            "other",
            "cycle-a",
            "cycle-b",
            "nested-a/entry",
            "nested-a/origin",
            "nested-b/entry",
            "nested-b/origin",
        ],
    )
}

fn prepared_fixture(name: &str, names: &[&str]) -> FileIdMap<CheckFileFacts> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture")
        .join(name)
        .canonicalize()
        .unwrap();
    names
        .iter()
        .map(|name| {
            let path = root.join(format!("{name}.mts"));
            let source = std::fs::read_to_string(&path).unwrap();
            let file = crate::ast::with_program(&path, &source, |program, _| {
                let profiles = [
                    EmbeddedSqlOptions::configured("@app/db", &[]),
                    EmbeddedSqlOptions::configured("@app/other", &[]),
                ];
                CheckFileFacts {
                    ts: Arc::new(collect_file_facts_from_program(
                        &path,
                        TsFactPlan {
                            imports: true,
                            module_bindings: true,
                            ..Default::default()
                        },
                        &TsFactContext::default(),
                        &source,
                        program,
                        None,
                        None,
                    )),
                    query_annotation: profiles
                        .iter()
                        .map(|options| {
                            (
                                options.clone(),
                                query_annotation::collect(program, &source, options),
                            )
                        })
                        .collect(),
                    embedded_sql: profiles
                        .iter()
                        .map(|options| {
                            (
                                options.clone(),
                                extract_embedded_sql_from_program(&path, program, &source, options),
                            )
                        })
                        .collect(),
                    source: Some(source.clone().into()),
                    ..Default::default()
                }
            })
            .unwrap();
            (path, file)
        })
        .collect()
}

#[path = "tests/demand_followups.rs"]
mod demand_followups;
#[path = "tests/external_roots.rs"]
mod external_roots;
#[path = "tests/profile_lifetimes.rs"]
mod profile_lifetimes;
#[cfg(feature = "test-instrumentation")]
#[path = "tests/project_scope.rs"]
mod project_scope;
#[cfg(feature = "test-instrumentation")]
#[path = "tests/stall_progress.rs"]
mod stall_progress;

#[path = "tests/demand_parity.rs"]
mod demand_parity;
#[path = "tests/legacy_demand.rs"]
mod legacy_demand;

#[path = "tests/demand_roots.rs"]
mod demand_roots;
#[path = "tests/profile_merges.rs"]
mod profile_merges;
#[path = "tests/target_reuse.rs"]
mod target_reuse;
