use super::*;
use super::{
    budget::Limits,
    catalog::PreparedCatalog,
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

pub(super) fn fixture() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-no-offset").join("low-helper-recovery")
}
pub(super) fn profile() -> crate::codebase::postgres::EmbeddedSqlOptions {
    crate::codebase::postgres::EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&[], &["QueryExecutor".into(), "QueryOptions".into()])
}
pub(super) fn prepared() -> FileIdMap<CheckFileFacts> {
    let mut files = FileIdMap::default();
    for name in [
        "helpers.mts",
        "query.mts",
        "binds.mts",
        "async.mts",
        "replay.mts",
        "partials.mts",
        "placements.mts",
        "precision.mts",
    ] {
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
            let syntax = super::super::collect(program, &ts, &source, &[profile()]);
            CheckFileFacts {
                ts: Arc::new(ts),
                sql_identifier_syntax: syntax,
                ..Default::default()
            }
        })
        .unwrap();
        files.insert(path, facts);
    }
    files
}
#[test]
fn helper_values_recover_fresh_builders_defaults_and_all_boolean_alternatives() {
    let files = prepared();
    let helper = fixture().join("src/helpers.mts");
    let query = fixture().join("src/query.mts");
    let resolve = |specifier: &str, _: &std::path::Path| {
        (specifier == "./helpers.mjs").then(|| helper.clone())
    };
    let catalog = PreparedCatalog {
        application: None,
        profile: None,
        files: &files,
        resolve: &resolve,
    };
    let source = std::fs::read_to_string(&query).unwrap();
    let mut cache = Cache::default();
    let mut recovered = 0;
    for call in &files
        .get(&query)
        .unwrap()
        .sql_identifier_syntax
        .recovery
        .calls
    {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if !raw.starts_with("read(")
            || raw.contains("dynamicFragment")
            || raw.contains("escapedBuilder")
        {
            continue;
        }
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.evaluate(&query, &call.arguments[0]).expect(raw);
        assert!(!values.is_empty());
        for value in &values {
            assert!(matches!(value, Value::Sql(_)), "{raw}");
        }
        if raw.contains("fragment({ enabled") {
            assert_eq!(values.len(), 2);
        }
        if raw.contains("offsetFragment") {
            let Value::Sql(sql) = &values[0] else {
                panic!()
            };
            let text = sql.raw.sql.clone();
            let offset = crate::codebase::postgres::sql_offset_facts(&text).unwrap()[0];
            let (path, line) = Recovered::position(&sql.raw, offset.line, offset.column, &query);
            assert_eq!(path, helper);
            assert!(std::fs::read_to_string(path)
                .unwrap()
                .lines()
                .nth(line - 1)
                .unwrap()
                .contains("OFFSET_ORIGIN"));
        }
        recovered += 1;
    }
    assert_eq!(recovered, 5);
}
#[test]
fn helper_values_reject_opaque_calls_and_dynamic_fragments() {
    let files = prepared();
    let helper = fixture().join("src/helpers.mts");
    let query = fixture().join("src/query.mts");
    let resolve = |specifier: &str, _: &std::path::Path| {
        (specifier == "./helpers.mjs").then(|| helper.clone())
    };
    let catalog = PreparedCatalog {
        application: None,
        profile: None,
        files: &files,
        resolve: &resolve,
    };
    let source = std::fs::read_to_string(&query).unwrap();
    let mut cache = Cache::default();
    let mut rejected = 0;
    for call in &files
        .get(&query)
        .unwrap()
        .sql_identifier_syntax
        .recovery
        .calls
    {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if raw.starts_with("read(")
            && (raw.contains("dynamicFragment") || raw.contains("escapedBuilder"))
        {
            assert!(
                Engine::new(&catalog, &mut cache, Limits::default())
                    .evaluate(&query, &call.arguments[0])
                    .is_none(),
                "{raw}"
            );
            rejected += 1;
        }
    }
    assert_eq!(rejected, 2);
}
#[test]
fn helper_budget_rejects_overflow_depth_variants_and_cache_warmth_bypasses() {
    use super::budget::{Budget, Cost, Exhausted};
    let mut budget = Budget::new(Limits {
        depth: 8,
        variants: 16,
        work: 3,
        sql_bytes: 128,
    });
    assert_eq!(
        budget.charge(
            Cost {
                work: 1,
                depth: 8,
                sql_bytes: 0
            },
            1
        ),
        Err(Exhausted::Depth)
    );
    assert_eq!(
        budget.charge(
            Cost {
                work: usize::MAX,
                depth: 1,
                sql_bytes: 0
            },
            0
        ),
        Err(Exhausted::Work)
    );
    assert_eq!(budget.variants(17), Err(Exhausted::Variants));
    assert!(budget.variants(16).is_ok());
    assert!(budget
        .charge(
            Cost {
                work: 3,
                sql_bytes: 128,
                depth: 1
            },
            0
        )
        .is_ok());
    assert_eq!(
        budget.charge(
            Cost {
                work: 1,
                depth: 1,
                sql_bytes: 0
            },
            0
        ),
        Err(Exhausted::Work)
    );
}

#[test]
fn generated_bind_ranges_keep_real_parameters_and_known_branches() {
    let files = prepared();
    let helper = fixture().join("src/helpers.mts");
    let path = fixture().join("src/binds.mts");
    let resolve = |specifier: &str, _: &std::path::Path| {
        (specifier == "./helpers.mjs").then(|| helper.clone())
    };
    let catalog = PreparedCatalog {
        application: None,
        profile: None,
        files: &files,
        resolve: &resolve,
    };
    let source = std::fs::read_to_string(&path).unwrap();
    let mut cache = Cache::default();
    let mut checked = 0;
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
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.evaluate(&path, &call.arguments[0]).expect(raw);
        let Value::Sql(sql) = &values[0] else {
            panic!()
        };
        assert!(!sql
            .raw
            .sql
            .contains(crate::codebase::postgres::embedded::INTERNAL_PLACEHOLDER));
        if raw.contains("nestedBind") {
            assert!(!sql.raw.sql.contains("OFFSET"));
        }
        if raw.contains("invalidBoundRelation") {
            assert!(crate::codebase::postgres::parse_postgres_sql(&sql.raw.sql).is_err());
        }
        if raw.contains("AS ten") {
            assert!(sql.raw.sql.contains("$10 AS ten"));
            let offset = crate::codebase::postgres::sql_offset_facts(&sql.raw.sql).unwrap()[0];
            let (origin, line) = Recovered::position(&sql.raw, offset.line, offset.column, &path);
            assert_eq!(origin, path);
            assert!(source
                .lines()
                .nth(line - 1)
                .unwrap()
                .contains("OFFSET_TENTH_BIND"));
        }
        if raw.contains("mixedFragment") {
            assert!(engine.residual());
            assert_eq!(
                crate::codebase::postgres::sql_offset_facts(&sql.raw.sql)
                    .unwrap()
                    .len(),
                1
            );
        } else {
            assert!(!engine.residual());
        }
        checked += 1;
    }
    assert_eq!(checked, 5);
}

#[test]
fn typed_helper_transport_checks_every_use_and_real_sql_sink() {
    let files = prepared();
    let helper = fixture().join("src/helpers.mts");
    let path = fixture().join("src/query.mts");
    let profile = profile();
    let resolve = |specifier: &str, _: &std::path::Path| {
        (specifier == "./helpers.mjs").then(|| helper.clone())
    };
    let catalog = PreparedCatalog {
        application: None,
        files: &files,
        profile: Some(&profile),
        resolve: &resolve,
    };
    let source = std::fs::read_to_string(&path).unwrap();
    let models = &files.get(&path).unwrap().sql_identifier_syntax.recovery;
    let executors = &models.executors[&profile];
    let mut cache = Cache::default();
    let mut checked = 0;
    for call in &models.calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if !["forward(", "executeOptions(", "escaped(", "recursive("]
            .iter()
            .any(|prefix| raw.starts_with(prefix))
        {
            continue;
        }
        let outcome = Engine::new(&catalog, &mut cache, Limits::default())
            .transport(&path, call, executors)
            .unwrap();
        if raw.starts_with("forward(") || raw.starts_with("executeOptions(") {
            assert!(outcome.complete(), "{raw}");
            assert_eq!(outcome.alternatives.len(), 1);
            assert!(outcome.alternatives[0].sql.contains("$1"));
        } else {
            assert!(outcome.residual, "{raw}");
        }
        checked += 1;
    }
    assert_eq!(checked, 4);
}
#[test]
fn aggregate_sql_bytes_and_cache_hits_obey_each_roots_limits() {
    let files = prepared();
    let helper = fixture().join("src/helpers.mts");
    let path = fixture().join("src/query.mts");
    let resolve = |specifier: &str, _: &std::path::Path| {
        (specifier == "./helpers.mjs").then(|| helper.clone())
    };
    let catalog = PreparedCatalog {
        application: None,
        files: &files,
        profile: None,
        resolve: &resolve,
    };
    let source = std::fs::read_to_string(&path).unwrap();
    let call = files
        .get(&path)
        .unwrap()
        .sql_identifier_syntax
        .recovery
        .calls
        .iter()
        .find(|call| {
            source[call.span.0 as usize..call.span.1 as usize].starts_with("read(")
                && source[call.span.0 as usize..call.span.1 as usize].contains("fixedFragment")
        })
        .unwrap();
    let mut warm = Cache::default();
    assert!(Engine::new(&catalog, &mut warm, Limits::default())
        .evaluate(&path, &call.arguments[0])
        .is_some());
    for mut cache in [Cache::default(), warm] {
        let limits = Limits {
            sql_bytes: 8,
            ..Limits::default()
        };
        let mut engine = Engine::new(&catalog, &mut cache, limits);
        assert!(engine.evaluate(&path, &call.arguments[0]).is_none());
        assert!(engine.residual());
    }
}

#[test]
fn async_helpers_keep_promise_values_distinct_and_unknown_await_residual() {
    let files = prepared();
    let helper = fixture().join("src/helpers.mts");
    let path = fixture().join("src/async.mts");
    let profile = profile();
    let resolve = |specifier: &str, _: &std::path::Path| {
        (specifier == "./helpers.mjs").then(|| helper.clone())
    };
    let effects = PreparedCatalog {
        application: None,
        files: &files,
        profile: Some(&profile),
        resolve: &resolve,
    };
    let pure = PreparedCatalog {
        application: None,
        files: &files,
        profile: None,
        resolve: &resolve,
    };
    let source = std::fs::read_to_string(&path).unwrap();
    let models = &files.get(&path).unwrap().sql_identifier_syntax.recovery;
    let executors = &models.executors[&profile];
    let mut checked = 0;
    for call in &models.calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        let mut cache = Cache::default();
        if raw.starts_with("read(") {
            let mut engine = Engine::new(&pure, &mut cache, Limits::default());
            let values = engine.evaluate(&path, &call.arguments[0]);
            assert_eq!(
                values.is_some(),
                raw.contains("await asyncFragment"),
                "{raw}"
            );
            checked += 1;
        } else if ["asyncForward(", "asyncCarrier(", "unknownAwait("]
            .iter()
            .any(|prefix| raw.starts_with(prefix))
        {
            let outcome = Engine::new(&effects, &mut cache, Limits::default())
                .transport(&path, call, executors)
                .unwrap();
            assert_eq!(
                outcome.complete(),
                raw.starts_with("asyncForward("),
                "{raw}"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 5);
}

#[test]
fn returned_builder_payload_is_an_executor_carrier_until_a_checked_sink_consumes_it() {
    let files = prepared();
    let helper = fixture().join("src/helpers.mts");
    let path = fixture().join("src/async.mts");
    let profile = profile();
    let resolve = |specifier: &str, _: &std::path::Path| {
        (specifier == "./helpers.mjs").then(|| helper.clone())
    };
    let catalog = PreparedCatalog {
        application: None,
        files: &files,
        profile: Some(&profile),
        resolve: &resolve,
    };
    let source = std::fs::read_to_string(&path).unwrap();
    let models = &files.get(&path).unwrap().sql_identifier_syntax.recovery;
    let mut checked = 0;
    for call in &models.calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if ![
            "builderCarrier(",
            "nestedBuilderCarrier(",
            "consumeBuilderBind(",
        ]
        .iter()
        .any(|prefix| raw.starts_with(prefix))
        {
            continue;
        }
        let mut cache = Cache::default();
        let proof = Engine::new(&catalog, &mut cache, Limits::default())
            .transport(&path, call, &models.executors[&profile])
            .unwrap();
        assert_eq!(
            proof.complete(),
            raw.starts_with("consumeBuilderBind("),
            "{raw}"
        );
        checked += 1;
    }
    assert_eq!(checked, 3);
}
