use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    tests::{fixture, profile},
    value::Value,
    values::{Cache, Engine},
};
use crate::codebase::{
    check_facts::CheckFileFacts,
    ts_source::{
        facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
        FileIdMap,
    },
};
use std::{path::PathBuf, sync::Arc};

fn prepared() -> (PathBuf, String, FileIdMap<CheckFileFacts>) {
    let path = fixture().join("src/predicate-refinement.mts");
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
        CheckFileFacts {
            sql_identifier_syntax: super::super::collect(program, &ts, &source, &[profile()]),
            ts: Arc::new(ts),
            ..Default::default()
        }
    })
    .unwrap();
    let mut files = FileIdMap::default();
    files.insert(path.clone(), facts);
    (path, source, files)
}

fn sql(value: &Value) -> &str {
    match value {
        Value::Sql(text) | Value::Text(text) => &text.raw.sql,
        _ => panic!("expected recovered SQL value"),
    }
}

#[test]
fn strict_scalar_predicates_correlate_boolean_builder_branches_within_budget() {
    let (path, source, files) = prepared();
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &|_, _| None,
    };
    let mut checked = [false; 3];
    for call in &files[&path].sql_identifier_syntax.recovery.calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if !raw.starts_with("read(") {
            continue;
        }
        let label = source[call.span.1 as usize..].lines().next().unwrap();
        let mut cache = Cache::default();
        let mut engine = Engine::new(
            &catalog,
            &mut cache,
            Limits {
                variants: 4,
                ..Limits::default()
            },
        );
        let values = engine.evaluate(&path, &call.arguments[0]);
        if label.contains("CORRELATED") {
            let values = values.expect(raw);
            assert!(!engine.residual(), "{raw}");
            assert_eq!(values.len(), 3, "{raw}");
            let statements = values
                .iter()
                .map(sql)
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(statements.len(), 3, "{raw}");
            checked[0] = true;
        } else if label.contains("KEEP_DROP") {
            let values = values.expect(raw);
            assert!(!engine.residual(), "{raw}");
            assert!(
                values.iter().any(|value| sql(value).contains("DROP TABLE")),
                "{raw}"
            );
            assert!(
                values.iter().any(|value| sql(value).contains("SELECT id")),
                "{raw}"
            );
            checked[1] = true;
        } else if label.contains("COERCIVE") {
            assert!(engine.residual(), "loose equality must not refine: {raw}");
            checked[2] = true;
        }
    }
    assert_eq!(checked, [true; 3]);
}
