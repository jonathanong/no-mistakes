use super::super::File;
use super::state_test_support::{assert_same_state, new};
use crate::codebase::{
    postgres::{
        extract_embedded_sql_from_program,
        query_annotation::{self, modules::Module},
        EmbeddedSqlOptions,
    },
    ts_source::facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[test]
fn unmodeled_calls_precede_export_tasks_with_imported_state_rollback() {
    for precise in [false, true] {
        let directory = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
            .join("helper-tracing-entrypoint-checkpoints/imports");
        let root = directory.join("root.mts");
        let shared = directory.join("shared.mts");
        let collect = |path: &PathBuf| {
            let source = std::fs::read_to_string(path).unwrap();
            let options = EmbeddedSqlOptions::configured("@app/db", &[]);
            let (mut facts, ts, embedded) =
                crate::ast::with_program(path, &source, |program, _| {
                    (
                        query_annotation::collect(program, &source, &options),
                        collect_file_facts_from_program(
                            path,
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
                        ),
                        extract_embedded_sql_from_program(path, program, &source, &options),
                    )
                })
                .unwrap();
            facts.precise_records = precise;
            (path.clone(), facts, ts, embedded)
        };
        let collected = [collect(&root), collect(&shared)];
        let files = collected
            .iter()
            .map(|(path, facts, ts, embedded)| {
                (
                    path.clone(),
                    File {
                        facts,
                        module: Arc::new(Module::new(Arc::new(ts.clone()))),
                        executors: embedded.call_spans.iter().copied().collect(),
                    },
                )
            })
            .collect();
        let resolve = |specifier: &str, from: &Path| {
            (specifier == "./shared.mjs" && from == root.as_path()).then(|| shared.clone())
        };
        let mut old = new(&files, resolve, false);
        let mut candidate = new(&files, resolve, false);
        let tasks = super::tasks::Tasks::new(&files[&root]).count();
        assert!(
            tasks >= 2,
            "fixture must exercise calls plus independent function tasks"
        );
        super::operations::reset();
        old.run_reference(&root);
        assert_eq!(
            super::operations::snapshot(),
            super::operations::Operations {
                captures: 1,
                restores: tasks,
                fresh: tasks
            }
        );
        super::operations::reset();
        candidate.run(&root);
        assert_eq!(
            super::operations::snapshot(),
            super::operations::Operations {
                captures: 1,
                restores: tasks - 1,
                fresh: tasks
            }
        );
        assert!(!candidate.events.is_empty());
        assert_same_state(&old, &candidate);
    }
}
