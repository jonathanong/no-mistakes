use super::Scope;
use crate::codebase::postgres::query_annotation::{evaluate, modules::Module};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub(in crate::codebase::postgres::query_annotation::evaluate) fn load(
    name: &str,
) -> (PathBuf, Scope) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../fixtures/performance/postgres-scope-frontier/{name}.mts"
    ));
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@frontier/db", &[]),
        )
    })
    .unwrap();
    let files = FxHashMap::from_iter([(
        path.clone(),
        evaluate::File {
            facts: &facts,
            module: Arc::new(Module::new(Arc::new(TsFileFacts::default()))),
            executors: Default::default(),
            initial_template: Default::default(),
        },
    )]);
    let mut evaluator = evaluate::new(&files, |_: &str, _: &Path| None, false);
    let env = evaluator.module_environment(&path);
    (path, evaluator.scopes[env].clone())
}
