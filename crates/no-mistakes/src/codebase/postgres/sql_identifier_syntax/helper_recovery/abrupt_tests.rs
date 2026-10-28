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
    let root = fixture().join("src");
    let mut files = FileIdMap::default();
    let mut main = String::new();
    for name in [
        "abrupt-proof.mts",
        "abrupt-proof-helper.mts",
        "abrupt-escape-helper.mts",
        "abrupt-shadowed-helper.mts",
    ] {
        let path = root.join(name);
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
        if name == "abrupt-proof.mts" {
            main = source;
        }
        files.insert(path, facts);
    }
    (root.join("abrupt-proof.mts"), main, files)
}

#[test]
fn value_only_abrupt_paths_keep_source_effects_and_async_uncertainty() {
    let (path, source, files) = prepared();
    let root = fixture().join("src");
    let profile = profile();
    let resolve = |specifier: &str, _: &std::path::Path| -> Option<PathBuf> {
        match specifier {
            "./abrupt-proof-helper.mjs" => Some(root.join("abrupt-proof-helper.mts")),
            "./abrupt-escape-helper.mjs" => Some(root.join("abrupt-escape-helper.mts")),
            "./abrupt-shadowed-helper.mjs" => Some(root.join("abrupt-shadowed-helper.mts")),
            _ => None,
        }
    };
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &resolve,
    };
    let mut checked = 0;
    for call in &files[&path].sql_identifier_syntax.recovery.calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if !raw.starts_with("read(") {
            continue;
        }
        let label = source[call.span.1 as usize..].lines().next().unwrap();
        let mut cache = Cache::default();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.evaluate_at(&path, &call.arguments[0], Some(call.span));
        if label.contains("COMPLETE") {
            assert!(!engine.residual(), "{raw}");
            let values = values.unwrap();
            assert_eq!(
                values.len(),
                if raw.contains("selected(mode)") { 2 } else { 1 },
                "{raw}"
            );
            assert!(
                values.iter().all(|value| matches!(value, Value::Sql(_))),
                "{raw}"
            );
        } else {
            assert!(
                engine.residual()
                    || values.as_ref().is_none_or(|values| {
                        !values
                            .iter()
                            .all(|value| matches!(value, Value::Sql(_) | Value::Text(_)))
                    }),
                "{raw}"
            );
        }
        checked += 1;
    }
    assert_eq!(checked, 5);
}

#[test]
fn site_stop_cache_cannot_hide_later_transport_sql_or_throw_effects() {
    let (path, source, files) = prepared();
    let profile = profile();
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &|_, _| None,
    };
    let models = &files[&path].sql_identifier_syntax.recovery;
    let call = |prefix: &str| {
        models
            .calls
            .iter()
            .find(|call| source[call.span.0 as usize..call.span.1 as usize].starts_with(prefix))
            .unwrap()
    };
    let executors = &models.executors[&profile];
    let mut cache = Cache::default();
    let first = Engine::new(&catalog, &mut cache, Limits::default())
        .site(&path, call("query('SELECT 1')"), executors)
        .unwrap();
    assert_eq!(first.alternatives.len(), 1);
    let complete = Engine::new(&catalog, &mut cache, Limits::default())
        .transport(&path, call("twoQueries(query)"), executors)
        .unwrap();
    assert!(complete
        .alternatives
        .iter()
        .any(|sql| sql.sql.contains("DROP TABLE later")));
    assert_eq!(complete.alternatives.len(), 2);
    let thrown = Engine::new(&catalog, &mut cache, Limits::default())
        .transport(&path, call("thrownSql(query)"), executors)
        .unwrap();
    assert!(thrown
        .alternatives
        .iter()
        .any(|sql| sql.sql.contains("DROP TABLE thrown")));
    let escaped = Engine::new(&catalog, &mut cache, Limits::default())
        .transport(&path, call("thrownExecutor(query)"), executors)
        .unwrap();
    assert!(escaped.residual);
    let opaque = Engine::new(&catalog, &mut cache, Limits::default())
        .transport(&path, call("opaqueThrow(query)"), executors)
        .unwrap();
    assert!(opaque.residual);
    for constructor in ["new Constructor(query)", "new asyncHelper(query)"] {
        let model = call(constructor);
        assert!(model.construction, "{constructor}");
        assert!(
            Engine::new(&catalog, &mut cache, Limits::default())
                .transport(&path, model, executors)
                .is_none(),
            "{constructor}"
        );
    }
}

#[test]
fn invalidated_constructor_argument_retains_source_sql_effect() {
    use crate::codebase::postgres::embedded::helpers::program::Stmt;
    let path = fixture().join("src/abrupt-invalid-tag.mts");
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
    let profile = profile();
    let models = &facts.sql_identifier_syntax.recovery;
    let constructor = models
        .calls
        .iter()
        .find(|call| {
            source[call.span.0 as usize..call.span.1 as usize].starts_with("new sourceCtor(")
        })
        .unwrap();
    assert!(constructor.construction);
    assert!(constructor.callee.is_some());
    assert!(matches!(
        constructor.arguments[0].program.body.as_slice(),
        [Stmt::Opaque]
    ));
    let sink = models
        .calls
        .iter()
        .find(|call| {
            source[call.span.0 as usize..call.span.1 as usize].starts_with("read(selected(")
        })
        .unwrap()
        .clone();
    let mut files = FileIdMap::default();
    files.insert(path.clone(), facts);
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &|_, _| None,
    };
    let mut cache = Cache::default();
    let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
    let values = engine
        .evaluate_at(&path, &sink.arguments[0], Some(sink.span))
        .unwrap();
    assert!(values
        .iter()
        .any(|value| matches!(value, Value::Text(sql) if sql.raw.sql == "SELECT 1")));
    assert!(
        engine.residual(),
        "constructor's source-visible SQL call must remain an obligation"
    );
}
