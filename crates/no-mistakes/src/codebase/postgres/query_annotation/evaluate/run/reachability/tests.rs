mod reference;
use super::*;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use reference::exported_reaches;
use std::path::PathBuf;

#[test]
fn prepared_callable_wrappers_preserve_existing_reachability() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/executed-source/control-flow-returned-entrypoint-wrappers.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        )
    })
    .unwrap();
    // These internal wrappers keep their established traversal contract. Source
    // callable identities come from the same stored fixture as the public tests.
    let opaque = Expr::OpaqueCallback(Box::new(Expr::Name("wrapped".into())));
    assert!(wrapper_reaches(
        &facts.globals,
        &opaque,
        "runArray",
        &mut BTreeSet::new(),
        &[]
    ));
    let computed = Expr::ComputedMember(
        Box::new(Expr::Name("missing".into())),
        Box::new(Expr::Name("runArray".into())),
    );
    assert!(wrapper_reaches(
        &facts.globals,
        &computed,
        "runArray",
        &mut BTreeSet::new(),
        &[]
    ));
    assert!(!wrapper_reaches(
        &facts.globals,
        &computed,
        "missingTarget",
        &mut BTreeSet::new(),
        &[]
    ));
    let append = Expr::Append(
        Box::new(Expr::Name("missing".into())),
        Box::new(Expr::Name("runArray".into())),
    );
    assert!(wrapper_reaches(
        &facts.globals,
        &append,
        "runArray",
        &mut BTreeSet::new(),
        &[]
    ));
}

#[test]
fn prepared_branch_shadows_do_not_escape_global_callback() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/postgres-facts/executed-source/control-flow-returned-entrypoint-static-branches.ts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        )
    })
    .unwrap();
    for name in [
        "shadowedArm",
        "hoistedShadow",
        "parameterShadow",
        "selfShadow",
        "closureShadow",
    ] {
        assert!(
            !export_reachable_globals(&facts.globals, [name]).contains("contextual"),
            "{name}: {:?}",
            facts.globals[name]
        );
    }
    assert!(
        matches!(facts.globals["contextual"], Expr::Function(_)),
        "{:?}",
        facts.globals["contextual"]
    );
    let Expr::Function(hoisted) = &facts.globals["hoistedShadow"] else {
        panic!("stored hoisted shadow must retain its prepared function");
    };
    assert!(shadowed(hoisted, "contextual"));
}

fn prepared_fixture(name: &str) -> query_annotation::QueryAnnotationFileFacts {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/executed-source")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        )
    })
    .unwrap()
}

#[test]
fn shared_export_cycle_visits_each_global_once() {
    let facts = prepared_fixture("export-reachability-shared-cycle.ts");
    visits::reset();
    let reachable = export_reachable_globals(
        &facts.globals,
        facts
            .globals
            .keys()
            .filter(|name| name.starts_with("entry"))
            .map(String::as_str),
    );
    assert_eq!(
        facts
            .globals
            .keys()
            .filter(|name| reachable.contains(name.as_str()))
            .count(),
        68
    );
    assert_eq!(visits::count(), 68);
    assert!(export_reachable_globals(&facts.globals, ["directShadow"]).contains("leaf"));
    assert!(export_reachable_globals(&facts.globals, ["globalScopeReset"]).contains("leaf"));
    assert!(!export_reachable_globals(&facts.globals, ["nestedScope"]).contains("leaf"));
}

#[test]
fn union_matches_every_historical_export_target_pair() {
    for fixture in [
        "export-reachability-shared-cycle.ts",
        "export-reachability-single-export.ts",
        "control-flow-returned-entrypoint-wrappers.ts",
        "control-flow-returned-entrypoint-static-branches.ts",
    ] {
        let facts = prepared_fixture(fixture);
        for exported in facts.globals.keys() {
            let reachable = export_reachable_globals(&facts.globals, [exported.as_str()]);
            for target in facts.globals.keys().map(String::as_str) {
                assert_eq!(
                    reachable.contains(target),
                    exported_reaches(&facts.globals, exported, target, &mut BTreeSet::new()),
                    "{fixture}: {exported} -> {target}"
                );
            }
        }
    }
}

// Fresh traversal state each iteration, prepared physical fixture shared by both algorithms.
#[test]
#[ignore = "focused matched performance measurement"]
fn measure_export_reachability() {
    for (fixture, expected) in [
        ("export-reachability-shared-cycle.ts", 68),
        ("export-reachability-single-export.ts", 1),
    ] {
        let facts = prepared_fixture(fixture);
        let exports: Vec<_> = facts
            .globals
            .keys()
            .filter(|name| name.starts_with("entry"))
            .map(String::as_str)
            .collect();
        let iterations = if expected == 1 { 100_000 } else { 100 };
        for optimized in [false, true, true, false] {
            let start = std::time::Instant::now();
            for _ in 0..iterations {
                let globals = std::hint::black_box(&facts.globals);
                let exports = std::hint::black_box(exports.as_slice());
                let selected = if optimized {
                    let reachable = export_reachable_globals(globals, exports.iter().copied());
                    globals
                        .keys()
                        .filter(|name| reachable.contains(name.as_str()))
                        .count()
                } else {
                    globals
                        .keys()
                        .map(String::as_str)
                        .filter(|target| {
                            exports.iter().any(|exported| {
                                exported_reaches(globals, exported, target, &mut BTreeSet::new())
                            })
                        })
                        .count()
                };
                assert_eq!(std::hint::black_box(selected), expected);
            }
            eprintln!(
                "fixture={fixture}, optimized={optimized}, iterations={iterations}, elapsed={:?}",
                start.elapsed()
            );
        }
    }
}

fn wrapper_reaches<'a>(
    globals: &'a BTreeMap<String, Expr>,
    value: &'a Expr,
    target: &str,
    reachable: &mut BTreeSet<&'a str>,
    scopes: &[&Function],
) -> bool {
    value_reaches(globals, value, reachable, scopes);
    reachable.contains(target)
}

#[test]
fn fully_exported_globals_need_no_expression_traversal() {
    let facts = prepared_fixture("export-reachability-single-export.ts");
    assert_eq!(facts.globals.len(), 1);
    visits::reset();
    let reachable = export_reachable_globals(&facts.globals, ["entry", "entry"]);
    assert!(matches!(reachable, ExportReachability::All));
    assert_eq!(visits::count(), 0);
}

// Every call performs a fresh public supplied-source analysis; source bytes alone are reused.
#[test]
#[ignore = "cold public API control observation"]
fn measure_single_export_request() {
    use crate::codebase::postgres::{
        extract_executed_postgres_sql_with_imports, ExecutedPostgresSource,
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/executed-source/export-reachability-single-export.ts");
    let sources = [ExecutedPostgresSource {
        file_name: path.to_string_lossy().into_owned(),
        source: std::fs::read_to_string(&path).unwrap(),
    }];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    pool.install(|| {
        for block in 0..4 {
            let start = std::time::Instant::now();
            for _ in 0..100 {
                let rows =
                    extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap();
                assert!(std::hint::black_box(rows).is_empty());
            }
            eprintln!(
                "cold_request_block={block}, jobs=1, iterations=100, elapsed={:?}",
                start.elapsed()
            );
        }
    });
}

#[test]
fn empty_and_unexported_single_globals_need_no_expression_traversal() {
    visits::reset();
    assert!(matches!(
        export_reachable_globals(&BTreeMap::new(), []),
        ExportReachability::All
    ));
    let facts = prepared_fixture("export-reachability-single-export.ts");
    let selected = export_reachable_globals(&facts.globals, ["missing"]);
    assert!(!selected.contains("entry"));
    assert_eq!(visits::count(), 0);
}
