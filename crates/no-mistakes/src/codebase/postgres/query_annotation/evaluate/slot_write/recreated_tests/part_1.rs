use super::super::super::{File, Value};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{
    EmbeddedSqlOptions, extract_embedded_sql_from_program, query_annotation,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn recreated_deleted_slot_callback_runs_only_after_the_arguments_object_escapes() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-deleted-slot-callback/src/query.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    let ts = TsFileFacts::default();
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: std::sync::Arc::new(Module::new(ts.clone().into())),
            executors: Default::default(),
        },
    )]);
    let mut evaluator = setup::evaluator(&files);
    let root = evaluator.module_environment(&path);
    let call = |name: &str, sql: &str| query_annotation::Expr::Call {
        callee: Box::new(query_annotation::Expr::Name(name.into())),
        args: vec![query_annotation::Expr::Text(sql.into())],
        start: 0,
        end: 0,
    };

    let no_escape = evaluator.expr(
        &call("recreatedCallbackWithoutEscape", "/* no escape */ SELECT 1"),
        &path,
        &root,
        16,
        false,
    );
    assert!(matches!(
        no_escape,
        Value::Prefix(text, true, None) if text == "/* no escape */ SELECT 1"
    ));

    let later_escape = evaluator.expr(
        &call("recreatedCallbackThenEscape", "/* later escape */ SELECT 1"),
        &path,
        &root,
        16,
        false,
    );
    assert!(matches!(later_escape, Value::Unknown));
}
