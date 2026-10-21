use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::{extract_embedded_sql_from_program, EmbeddedSqlOptions};
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};

pub(super) fn fixture(name: &str) -> (std::path::PathBuf, String, CheckFileFacts) {
    let path = support::root()
        .parent()
        .unwrap()
        .join("helper-tracing-arm-parallel-factories")
        .join("src")
        .join(if name.contains('.') {
            name.to_string()
        } else {
            format!("{name}.mts")
        });
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let file = crate::ast::with_program(&path, &source, |program, _| {
        let facts = query_annotation::collect(program, &source, &options);
        CheckFileFacts {
            ts: std::sync::Arc::new(collect_file_facts_from_program(
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
            query_annotation: vec![(options.clone(), facts)],
            embedded_sql: vec![(
                options.clone(),
                extract_embedded_sql_from_program(&path, program, &source, &options),
            )],
            source: Some(source.clone().into()),
            ..Default::default()
        }
    })
    .unwrap();
    (path, source, file)
}
