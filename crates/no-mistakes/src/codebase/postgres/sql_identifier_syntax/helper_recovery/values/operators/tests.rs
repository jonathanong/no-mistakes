use super::super::super::{
    budget::Limits,
    catalog::PreparedCatalog,
    value::Value,
    values::{Cache, Engine},
};
use crate::codebase::{
    check_facts::CheckFileFacts,
    postgres::{sql_identifier_syntax::collect, EmbeddedSqlOptions},
    ts_source::{
        facts::{collect_file_facts_from_program, TsFactContext, TsFactPlan},
        FileIdMap,
    },
};
use std::{path::PathBuf, sync::Arc};

fn fixture() -> (PathBuf, String, FileIdMap<CheckFileFacts>) {
    prepared("operator-coverage.mts")
}
fn prepared(name: &str) -> (PathBuf, String, FileIdMap<CheckFileFacts>) {
    let path = crate::test_support::rule_fixture_root("postgres-no-offset")
        .join("low-helper-recovery/src")
        .join(name);
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
        let syntax = collect(
            program,
            &ts,
            &source,
            &[EmbeddedSqlOptions::configured("@example/db", &[])
                .with_scoped_executors(&[], &["QueryExecutor".into()])],
        );
        CheckFileFacts {
            ts: Arc::new(ts),
            sql_identifier_syntax: syntax,
            ..Default::default()
        }
    })
    .unwrap();
    let mut files = FileIdMap::default();
    files.insert(path.clone(), facts);
    (path, source, files)
}
#[test]
fn closed_operator_values_preserve_sql_and_decline_unknown_coercion() {
    let (path, source, files) = fixture();
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &|_, _| None,
    };
    let mut checked = 0;
    let mut cache = Cache::default();
    for call in &files
        .get(&path)
        .unwrap()
        .sql_identifier_syntax
        .recovery
        .calls
    {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if !raw.starts_with("read(") {
            continue;
        }
        let suffix = source[call.span.1 as usize..].lines().next().unwrap();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.evaluate(&path, &call.arguments[0]);
        if suffix.contains("UNKNOWN") {
            assert!(values.is_none());
            assert!(engine.residual());
        } else {
            assert!(!engine.residual(), "{raw}");
            let offset = values
                .unwrap()
                .into_iter()
                .map(|value| {
                    let sql = match value {
                        Value::Text(text) | Value::Sql(text) => text.raw.sql,
                        _ => panic!("{raw}"),
                    };
                    crate::codebase::postgres::sql_offset_facts(&sql)
                        .unwrap()
                        .len()
                })
                .sum::<usize>();
            assert_eq!(offset > 0, suffix.contains("OFFSET"), "{raw}");
        }
        checked += 1;
    }
    assert_eq!(checked, 16);
}
#[test]
fn concatenation_exhaustion_cannot_publish_a_complete_sql_value() {
    let (path, source, files) = fixture();
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &|_, _| None,
    };
    let call = files
        .get(&path)
        .unwrap()
        .sql_identifier_syntax
        .recovery
        .calls
        .iter()
        .find(|call| source[call.span.0 as usize..call.span.1 as usize].starts_with("read(joined("))
        .unwrap();
    for (limit, succeeds) in [(20, false), (35, false), (128, true)] {
        let mut cache = Cache::default();
        let mut engine = Engine::new(
            &catalog,
            &mut cache,
            Limits {
                sql_bytes: limit,
                ..Limits::default()
            },
        );
        assert_eq!(
            engine.evaluate(&path, &call.arguments[0]).is_some(),
            succeeds
        );
        assert_eq!(engine.residual(), !succeeds);
    }
}

#[test]
fn helper_returned_sql_is_not_fresh_when_an_incoming_statement_can_alias_it() {
    let (path, source, files) = prepared("alias-boundary.mts");
    let profile = files.get(&path).unwrap().sql_identifier_syntax.profiles[0]
        .0
        .clone();
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &|_, _| None,
    };
    let models = &files.get(&path).unwrap().sql_identifier_syntax.recovery;
    let mut checked = 0;
    for call in &models.calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        let mut cache = Cache::default();
        if ["unsafeReadonly(read,", "unsafeNested(read,"]
            .iter()
            .any(|prefix| raw.starts_with(prefix))
        {
            let proof = Engine::new(&catalog, &mut cache, Limits::default())
                .transport(&path, call, &models.executors[&profile])
                .unwrap();
            assert!(proof.residual, "{raw}");
            assert!(!proof.complete());
            checked += 1;
        } else if raw.starts_with("readonlyQuery(read,")
            || raw.starts_with("readonlyFragment(read,")
        {
            let proof = Engine::new(&catalog, &mut cache, Limits::default())
                .transport(&path, call, &models.executors[&profile])
                .unwrap();
            assert!(proof.complete(), "{raw}");
            let offsets = proof
                .alternatives
                .iter()
                .map(|sql| {
                    crate::codebase::postgres::sql_offset_facts(&sql.sql)
                        .unwrap()
                        .len()
                })
                .sum::<usize>();
            assert_eq!(offsets > 0, raw.starts_with("readonlyFragment"));
            checked += 1;
        } else if raw == "read(statement)" {
            let proof = Engine::new(&catalog, &mut cache, Limits::default()).site(
                &path,
                call,
                &models.executors[&profile],
            );
            let suffix = source[call.span.1 as usize..].lines().next().unwrap();
            if suffix.contains("FRESH_CLEAR") {
                assert!(proof.unwrap().complete());
            } else {
                assert!(proof.is_none_or(|proof| proof.residual));
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 6);
}
