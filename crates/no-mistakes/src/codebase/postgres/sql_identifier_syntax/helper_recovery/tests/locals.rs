use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    tests::{fixture, profile},
    value::Value,
    values::{Cache, Catalog, Engine},
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
fn exact_local_const_sql_operands_require_closed_uses_and_order() {
    let path = fixture().join("src/local-const.mts");
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
    let profile = profile();
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &|_, _| None,
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
        if raw == "read(updates['a'])" {
            let id = files[&path]
                .ts
                .module_bindings
                .as_ref()
                .unwrap()
                .bindings
                .iter()
                .find(|binding| binding.name == "updates")
                .unwrap()
                .id;
            assert!(
                Catalog::local(&catalog, &path, id, call.span).is_some(),
                "map const admission"
            );
        }
        let values = engine.evaluate_at(&path, &call.arguments[0], Some(call.span));
        if label.contains("COMPLETE") {
            assert!(!engine.residual(), "{raw}");
            assert!(
                values
                    .unwrap()
                    .iter()
                    .all(|value| matches!(value, Value::Sql(_))),
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
    assert_eq!(checked, 8);
}

#[test]
fn local_const_cannot_revive_reassigned_tag() {
    let path = fixture().join("src/local-const-tag-write.mts");
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
    let mut files = FileIdMap::default();
    files.insert(path.clone(), facts);
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &|_, _| None,
    };
    let call = files[&path]
        .sql_identifier_syntax
        .recovery
        .calls
        .iter()
        .find(|call| source[call.span.0 as usize..call.span.1 as usize].starts_with("read("))
        .unwrap();
    let mut cache = Cache::default();
    let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
    let values = engine.evaluate_at(&path, &call.arguments[0], Some(call.span));
    assert!(
        engine.residual()
            || values.as_ref().is_none_or(|values| {
                !values
                    .iter()
                    .all(|value| matches!(value, Value::Sql(_) | Value::Text(_)))
            })
    );
}

fn dynamic_fixture(name: &str) -> (std::path::PathBuf, String, FileIdMap<CheckFileFacts>) {
    let path = fixture().join("src").join(name);
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

#[test]
fn direct_eval_keeps_const_sql_map_uncertain() {
    let (path, source, files) = dynamic_fixture("local-const-eval.mts");
    let profile = profile();
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &|_, _| None,
    };
    let models = &files[&path].sql_identifier_syntax.recovery;
    assert!(models.dynamic_scope);
    let id = files[&path]
        .ts
        .module_bindings
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .find(|binding| binding.name == "map")
        .unwrap()
        .id;
    let call = models
        .calls
        .iter()
        .find(|call| {
            source[call.span.0 as usize..call.span.1 as usize].starts_with("read(map.sql)")
        })
        .unwrap();
    assert!(Catalog::local(&catalog, &path, id, call.span).is_none());
    let mut cache = Cache::default();
    let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
    let values = engine.evaluate_at(&path, &call.arguments[0], Some(call.span));
    assert!(
        engine.residual()
            || values.as_ref().is_none_or(|values| {
                !values
                    .iter()
                    .all(|value| matches!(value, Value::Sql(_) | Value::Text(_)))
            })
    );
}

#[test]
fn sloppy_with_statement_blocks_local_const_admission() {
    let (path, _, files) = dynamic_fixture("local-const-with.js");
    let models = &files[&path].sql_identifier_syntax.recovery;
    assert!(models.dynamic_scope);
    let id = files[&path]
        .ts
        .module_bindings
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .find(|binding| binding.name == "map")
        .unwrap()
        .id;
    let profile = profile();
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &|_, _| None,
    };
    assert!(Catalog::local(&catalog, &path, id, (u32::MAX, u32::MAX)).is_none());
}

#[test]
fn prior_executor_uses_cannot_expose_a_later_proven_builder() {
    let (path, source, files) = dynamic_fixture("local-const-escape.mts");
    let profile = profile();
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &|_, _| None,
    };
    let mut checked = 0;
    for call in &files[&path].sql_identifier_syntax.recovery.calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if !raw.starts_with("read(")
            || !source[call.span.1 as usize..]
                .lines()
                .next()
                .unwrap()
                .contains("CHECK")
        {
            continue;
        }
        let mut cache = Cache::default();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.evaluate_at(&path, &call.arguments[0], Some(call.span));
        assert!(
            engine.residual()
                || values.as_ref().is_none_or(|values| {
                    !values
                        .iter()
                        .all(|value| matches!(value, Value::Sql(_) | Value::Text(_)))
                }),
            "{raw}"
        );
        checked += 1;
    }
    assert_eq!(checked, 5);
}
