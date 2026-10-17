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

#[test]
fn targets_and_indexes_are_shared_across_profiles_and_parallel_evaluators() {
    let mut files = prepared();
    let candidates = files
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<BTreeSet<_>>();
    let calls = Mutex::new(BTreeMap::new());
    let observer = InvocationObserver::new(true);
    with_observer(Some(Arc::clone(&observer)), || {
        project(&mut files, |specifier, from| {
            *calls
                .lock()
                .unwrap()
                .entry((from.to_path_buf(), specifier.to_string()))
                .or_insert(0) += 1;
            let target = from.parent().unwrap().join(specifier).with_extension("mts");
            candidates.contains(&target).then_some(target)
        });
    });
    let calls = calls.into_inner().unwrap();
    assert!(
        calls.len() > 8,
        "exercise the full import/re-export catalog"
    );
    assert!(
        calls.values().all(|count| *count == 1),
        "every source/specifier, including failures, resolves once: {calls:?}"
    );
    assert!(calls
        .keys()
        .any(|(_, specifier)| specifier == "./missing.mjs"));
    // Known executor roots need no speculative resolution of unused imports.
    assert!(!calls
        .keys()
        .any(|(_, specifier)| specifier == "./unused.mjs" || specifier == "./type-only.mjs"));
    let snapshot = observer.snapshot();
    assert_eq!(snapshot.work["postgres.annotation.parallel_passes"], 1);
    assert_eq!(
        snapshot.work["postgres.annotation.modules"],
        candidates.len() as u64
    );
    assert_eq!(
        snapshot.work["postgres.annotation.target_resolutions"],
        calls.len() as u64
    );
    // The same relative string must retain both distinct importing sources.
    for (entry, expected) in [
        ("nested-a/entry.mts", "/* local */ SELECT 5"),
        ("nested-b/entry.mts", "SELECT 6"),
    ] {
        let (path, file) = files
            .iter()
            .find(|(path, _)| path.ends_with(entry))
            .unwrap();
        assert_eq!(calls[&(path.clone(), "./origin.mjs".into())], 1);
        let source = file.source.as_deref().unwrap();
        let call = "query(provide())";
        let start = source.find(call).unwrap() as u32;
        let facts = &file
            .query_annotation
            .iter()
            .find(|(options, _)| options.import_specifier == "@app/db")
            .unwrap()
            .1;
        assert_eq!(
            facts.calls[&(start, start + call.len() as u32)].as_deref(),
            Some(expected)
        );
    }
    let (path, file) = files
        .iter()
        .find(|(path, _)| path.ends_with("helper-tracing-target-reuse/entry.mts"))
        .unwrap();
    let source = file.source.as_deref().unwrap();
    for (options, facts) in &file.query_annotation {
        let executor = if options.import_specifier == "@app/db" {
            "query"
        } else {
            "other"
        };
        let expected = if executor == "query" {
            vec![
                ("safe()", Some("/* safe */ SELECT 1")),
                ("unsafe()", Some("SELECT 1")),
                ("shared()", Some("/* shared */ SELECT 2")),
                ("ambiguous()", None),
                ("missing()", None),
                ("cycled()", None),
                ("absentDefault()", None),
            ]
        } else {
            vec![
                ("safe()", Some("/* safe */ SELECT 1")),
                ("unsafe()", Some("SELECT 1")),
                ("missing()", None),
            ]
        };
        for (argument, expected) in expected {
            let call = format!("{executor}({argument})");
            let start = source.find(&call).unwrap() as u32;
            assert_eq!(
                facts
                    .calls
                    .get(&(start, start + call.len() as u32))
                    .map(|value| value.as_deref()),
                Some(expected),
                "{} {call}",
                path.display()
            );
        }
    }
}

#[test]
fn targets_do_not_escape_their_resolver_projection() {
    let mut files = prepared();
    let candidates = files
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<BTreeSet<_>>();
    let entry = candidates
        .iter()
        .find(|path| path.ends_with("helper-tracing-target-reuse/entry.mts"))
        .unwrap()
        .clone();
    let syntax = Arc::clone(&files[&entry].ts);
    let source = files[&entry].source.as_deref().unwrap();
    let call = "query(safe())";
    let start = source.find(call).unwrap() as u32;
    let span = (start, start + call.len() as u32);
    project(&mut files, |specifier, from| {
        let target = from.parent().unwrap().join(specifier).with_extension("mts");
        candidates.contains(&target).then_some(target)
    });
    assert_eq!(
        files[&entry].query_annotation[0].1.calls[&span].as_deref(),
        Some("/* safe */ SELECT 1")
    );
    let observer = InvocationObserver::new(true);
    // Reproject the exact retained facts. A previous successful target must
    // never answer a query owned by this new resolver/catalog projection.
    with_observer(Some(Arc::clone(&observer)), || {
        project(&mut files, |_, _| None)
    });
    assert!(observer.snapshot().work["postgres.annotation.target_resolutions"] > 0);
    assert_eq!(
        observer.snapshot().work["postgres.annotation.modules"],
        candidates.len() as u64
    );
    assert!(Arc::ptr_eq(&syntax, &files[&entry].ts));
    assert_eq!(files[&entry].query_annotation[0].1.calls[&span], None);
}

#[test]
fn demand_prunes_unrelated_modules_and_cycles_but_keeps_contextual_callers() {
    let mut files = prepared_fixture(
        "helper-tracing-demand-roots",
        &[
            "executor",
            "caller",
            "barrel",
            "entry",
            "cycle-a",
            "cycle-b",
            "unrelated",
            "unrelated-chain",
            "unrelated-cycle-a",
            "unrelated-cycle-b",
            "unresolved",
            "opaque-consumer",
            "computed",
            "dynamic",
            "required",
            "type-only",
        ],
    );
    for (_, file) in &mut files {
        file.query_annotation.truncate(1);
        file.embedded_sql.truncate(1);
    }
    let candidates = files.keys().cloned().collect::<BTreeSet<_>>();
    let resolve = |specifier: &str, from: &std::path::Path| {
        let target = from.parent().unwrap().join(specifier).with_extension("mts");
        candidates.contains(&target).then_some(target)
    };
    let collected = files
        .iter()
        .map(|(path, file)| {
            (
                path.clone(),
                super::File {
                    module: Arc::new(super::Module::new(Arc::clone(&file.ts))),
                    facts: &file.query_annotation[0].1,
                    executors: file.embedded_sql[0].1.call_spans.iter().copied().collect(),
                },
            )
        })
        .collect();
    let selected = super::demand::roots(
        &collected,
        &EmbeddedSqlOptions::configured("@app/db", &[]),
        &resolve,
    )
    .into_iter()
    .map(|path| path.file_stem().unwrap().to_str().unwrap())
    .collect::<BTreeSet<_>>();
    assert_eq!(
        selected,
        BTreeSet::from([
            "executor",
            "caller",
            "barrel",
            "entry",
            "cycle-a",
            "cycle-b",
            "unresolved",
            "opaque-consumer",
            "computed",
            "dynamic",
            "required",
        ])
    );
    drop(collected);
    let observer = InvocationObserver::new(true);
    let resolutions = Mutex::new(BTreeSet::new());
    with_observer(Some(Arc::clone(&observer)), || {
        project(&mut files, |specifier, from| {
            resolutions
                .lock()
                .unwrap()
                .insert((from.to_path_buf(), specifier.to_string()));
            let target = from.parent().unwrap().join(specifier).with_extension("mts");
            candidates.contains(&target).then_some(target)
        });
    });
    assert_eq!(observer.snapshot().work["postgres.annotation.roots"], 11);
    assert!(
        !resolutions
            .into_inner()
            .unwrap()
            .iter()
            .any(|(from, specifier)| {
                from.file_stem().unwrap() == "unresolved" && specifier == "./unrelated.mjs"
            }),
        "uncertain roots must not resolve their unused remaining imports"
    );
    let executor = files
        .iter()
        .find(|(path, _)| path.ends_with("executor.mts"))
        .unwrap()
        .1;
    let source = executor.source.as_deref().unwrap();
    let start = source.find("query(sql)").unwrap() as u32;
    // Executor-free importers must supply their contextual unannotated SQL.
    assert_eq!(
        executor.query_annotation[0].1.calls[&(start, start + 10)].as_deref(),
        Some("SELECT 1")
    );
}

#[test]
fn profile_batch_merges_in_the_same_order_with_one_or_four_workers() {
    let run = |threads| {
        let mut files = prepared();
        let candidates = files.keys().cloned().collect::<BTreeSet<_>>();
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                project(&mut files, |specifier, from| {
                    let target = from.parent().unwrap().join(specifier).with_extension("mts");
                    candidates.contains(&target).then_some(target)
                });
            });
        files
            .iter()
            .flat_map(|(path, file)| {
                file.query_annotation.iter().map(move |(profile, facts)| {
                    ((path.clone(), profile.clone()), facts.calls.clone())
                })
            })
            .collect::<BTreeMap<_, _>>()
    };
    // Contextual preference depends on root order; worker scheduling must not.
    assert_eq!(run(1), run(4));
}

#[test]
fn mixed_profile_scopes_match_separate_profile_projections() {
    let run = |only: Option<&str>| {
        let mut files = prepared();
        for (path, file) in &mut files {
            // Model distinct include scopes while retaining shared/cyclic helpers.
            file.query_annotation.retain(|(profile, _)| {
                let selected = if path.components().any(|part| part.as_os_str() == "nested-a") {
                    profile.import_specifier == "@app/db"
                } else if path.components().any(|part| part.as_os_str() == "nested-b") {
                    profile.import_specifier == "@app/other"
                } else {
                    true
                };
                selected && only.is_none_or(|name| profile.import_specifier == name)
            });
        }
        let candidates = files.keys().cloned().collect::<BTreeSet<_>>();
        project(&mut files, |specifier, from| {
            let target = from.parent().unwrap().join(specifier).with_extension("mts");
            candidates.contains(&target).then_some(target)
        });
        files
            .iter()
            .flat_map(|(path, file)| {
                file.query_annotation.iter().map(move |(profile, facts)| {
                    ((path.clone(), profile.clone()), facts.calls.clone())
                })
            })
            .collect::<BTreeMap<_, _>>()
    };
    let mut separate = run(Some("@app/db"));
    separate.extend(run(Some("@app/other")));
    assert_eq!(run(None), separate);
}

#[path = "tests/external_roots.rs"]
mod external_roots;
#[path = "tests/profile_lifetimes.rs"]
mod profile_lifetimes;
