use super::locate::{Located, Locator, MAX_REEXPORT_HOPS};
use super::{has_candidates, project};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, EmbeddedSqlKind, EmbeddedSqlOptions,
};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use crate::codebase::ts_source::{discover_visible_paths, FileIdMap};
use crate::diagnostics::{with_observer, InvocationObserver};
use crate::fx::FxHashSet;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[path = "tests/prerequisites.rs"]
mod prerequisites;

#[path = "tests/plain_clause.rs"]
mod plain_clause;

#[path = "tests/hasown.rs"]
mod hasown;

#[path = "tests/scalar_records.rs"]
mod scalar_records;

#[path = "tests/reopen_shapes.rs"]
mod reopen_shapes;

fn variant_root(name: &str) -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/variants")
            .join(name),
    )
}

/// Per-file facts exactly as the request pass would collect them, one parse per file.
fn prepared(name: &str) -> FileIdMap<CheckFileFacts> {
    let mut paths = discover_visible_paths(&variant_root(name).join("src"));
    paths.retain(|path| path.extension().is_some_and(|extension| extension == "mts"));
    paths.sort();
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    paths
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(&path).expect("fixture source");
            let file = crate::ast::with_program(&path, &source, |program, _| CheckFileFacts {
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
                postgres_file: crate::codebase::postgres::check_analysis::file::PostgresFileFacts {
                    embedded_sql: vec![(
                        options.clone(),
                        extract_embedded_sql_from_program(&path, program, &source, &options),
                    )],
                    ..Default::default()
                }
                .into_payload(),
                ..Default::default()
            })
            .expect("parse");
            (path, file)
        })
        .collect()
}

/// Resolves relative specifiers against the fixture's own files.
fn resolver<'a>(
    files: &FileIdMap<CheckFileFacts>,
    calls: &'a Mutex<BTreeMap<(PathBuf, String), u32>>,
) -> impl FnMut(&str, &Path) -> Option<PathBuf> + 'a {
    let known = files.keys().cloned().collect::<FxHashSet<_>>();
    move |specifier, from| {
        *calls
            .lock()
            .unwrap()
            .entry((from.to_path_buf(), specifier.to_string()))
            .or_default() += 1;
        let target = normalize_path(&from.parent()?.join(specifier));
        known.contains(&target).then_some(target)
    }
}

fn run(name: &str) -> FileIdMap<CheckFileFacts> {
    let mut files = prepared(name);
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    files
}

fn caller_calls(
    files: &FileIdMap<CheckFileFacts>,
) -> Vec<crate::codebase::postgres::EmbeddedSqlCall> {
    let (_, caller) = files
        .iter()
        .find(|(path, _)| path.ends_with("src/caller.mts"))
        .expect("caller");
    caller.postgres().embedded_sql[0].1.calls.clone()
}

#[path = "namespace_tests.rs"]
mod namespace_tests;

include!("tests_part1.rs");

include!("tests_part2.rs");
