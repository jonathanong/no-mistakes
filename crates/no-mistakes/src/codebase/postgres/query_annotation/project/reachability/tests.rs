use super::*;
use crate::codebase::postgres::{extract_embedded_sql_from_program, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};

#[test]
fn skips_disconnected_fixture_modules_but_keeps_cyclic_reexports() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-reachability/src");
    let names = [
        "root.mts",
        "cycle-a.mts",
        "cycle-b.mts",
        "barrel.mts",
        "helper.mts",
        "db.mts",
        "idle.mts",
    ];
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let prepared = names
        .iter()
        .map(|name| {
            let path = crate::codebase::ts_resolver::normalize_path(&root.join(name));
            let source = std::fs::read_to_string(&path).unwrap();
            let (ts, calls) = crate::ast::with_program(&path, &source, |program, _| {
                (
                    collect_file_facts_from_program(
                        &path,
                        TsFactPlan::imports(),
                        &TsFactContext::default(),
                        &source,
                        program,
                        None,
                        None,
                    ),
                    extract_embedded_sql_from_program(&path, program, &source, &options)
                        .call_starts,
                )
            })
            .unwrap();
            (path, ts, calls)
        })
        .collect::<Vec<_>>();
    let empty = crate::codebase::postgres::query_annotation::QueryAnnotationFileFacts::default();
    let files = prepared
        .iter()
        .map(|(path, ts, calls)| {
            (
                path.clone(),
                File {
                    facts: &empty,
                    ts,
                    executors: calls.iter().copied().collect(),
                    imports: fx_map(),
                    exports: fx_map(),
                },
            )
        })
        .collect::<FxHashMap<_, _>>();
    let roots = roots_reaching_executors(&files, &|specifier, from| {
        if specifier == "@app/db" {
            return Some(crate::codebase::ts_resolver::normalize_path(
                &root.join("db.mts"),
            ));
        }
        let local = specifier.strip_prefix("./")?.replace(".mjs", ".mts");
        Some(crate::codebase::ts_resolver::normalize_path(
            &from.with_file_name(local),
        ))
    });
    for name in names
        .iter()
        .filter(|name| **name != "idle.mts" && **name != "db.mts")
    {
        assert!(
            roots.contains(&crate::codebase::ts_resolver::normalize_path(
                &root.join(name)
            )),
            "{name}"
        );
    }
    assert!(
        !roots.contains(&crate::codebase::ts_resolver::normalize_path(
            &root.join("idle.mts")
        ))
    );
    assert!(
        !roots.contains(&crate::codebase::ts_resolver::normalize_path(
            &root.join("db.mts")
        ))
    );
    let only_local = roots_reaching_executors(&files, &|_, _| None);
    assert_eq!(
        only_local,
        FxHashSet::from_iter([crate::codebase::ts_resolver::normalize_path(
            &root.join("helper.mts")
        )])
    );
    let mut without_executors = files;
    for file in without_executors.values_mut() {
        file.executors.clear();
    }
    assert!(roots_reaching_executors(&without_executors, &|_, _| None).is_empty());
}
