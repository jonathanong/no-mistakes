use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    value::Value,
    values::{Cache, Engine},
    Recovered,
};
use crate::codebase::{
    check_facts::CheckFileFacts,
    ts_source::{
        facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
        FileIdMap,
    },
};
use std::sync::Arc;

#[test]
fn variant_limits_keep_known_sql_and_debug_output_private() {
    let path = super::tests::fixture().join("src/variant-limits.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        let ts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        let syntax = super::super::collect(program, &ts, &source, &[super::tests::profile()]);
        CheckFileFacts {
            ts: Arc::new(ts),
            sql_identifier_syntax: syntax,
            ..Default::default()
        }
    })
    .unwrap();
    let mut files = FileIdMap::default();
    files.insert(path.clone(), facts);
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &|_, _| None,
    };
    let call = files[&path]
        .sql_identifier_syntax
        .recovery
        .calls
        .iter()
        .find(|call| source[call.span.0 as usize..call.span.1 as usize].starts_with("read("))
        .unwrap();
    for variants in [1, 2] {
        let mut cache = Cache::default();
        let mut engine = Engine::new(
            &catalog,
            &mut cache,
            Limits {
                variants,
                ..Limits::default()
            },
        );
        let values = engine.evaluate(&path, &call.arguments[0]).unwrap();
        assert_eq!(values.len(), variants);
        assert_eq!(engine.residual(), variants == 1);
        let alternatives = values
            .into_iter()
            .map(|value| {
                let Value::Sql(sql) = value else {
                    panic!("saved branch must return SQL")
                };
                sql.raw
            })
            .collect::<Vec<_>>();
        assert!(alternatives.iter().any(|sql| sql.sql.contains("OFFSET 8")));
        let proof = Recovered {
            alternatives,
            residual: engine.residual(),
            nonexecuting: false,
        };
        let debug = format!("{:?}", super::super::Input::Recovered(Arc::new(proof)));
        assert!(debug.contains(&format!("alternatives: {variants}")));
        assert!(debug.contains(&format!("residual: {}", variants == 1)));
        assert!(!debug.contains("private-sql-payload"));
        assert!(!debug.contains("variant-limits.mts"));
        assert!(!debug.contains("source_path"));
    }
}
