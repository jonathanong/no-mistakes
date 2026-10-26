use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    patch_support_tests as support,
    value::Value,
    values::{Cache, Engine},
};

#[test]
fn saved_program_limits_fail_closed_without_losing_retained_sql() {
    let files = support::prepare(&["patch-limits.mts"]);
    let path = super::tests::fixture().join("src/patch-limits.mts");
    let resolve =
        |specifier: &str, from: &std::path::Path| support::resolve(&files, from, specifier);
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &resolve,
    };
    let mut observed_partial = false;
    let mut observed_exhaustion = false;
    let mut observed_complete = false;
    // Small limits exercise production guards without huge generated fixtures.
    for (name, arguments) in [
        ("conditionalBudget", vec![Value::Scalar]),
        ("templateBudget", vec![]),
        ("appendBudget", vec![]),
    ] {
        let key = catalog
            .binding(&path, support::binding(&files, &path, name))
            .unwrap();
        for work in 0..80 {
            for sql_bytes in [0, 16, 32, 64, 128, 4096] {
                let mut cache = Cache::default();
                let mut engine = Engine::new(
                    &catalog,
                    &mut cache,
                    Limits {
                        work,
                        sql_bytes,
                        ..Limits::default()
                    },
                );
                let values = engine.run(&key, &arguments).unwrap_or_default();
                observed_exhaustion |= values.is_empty();
                observed_partial |= !values.is_empty() && engine.residual();
                observed_complete |= !values.is_empty() && !engine.residual();
                for value in values {
                    let Value::Sql(text) = value else {
                        panic!("saved builder must return SQL");
                    };
                    assert!(text.raw.sql.contains("OFFSET 9"));
                    assert!(text.valid_binds());
                }
            }
        }
        let mut warm = Cache::default();
        assert!(Engine::new(&catalog, &mut warm, Limits::default())
            .run(&key, &arguments)
            .is_some());
        for limits in [
            Limits {
                work: 0,
                ..Limits::default()
            },
            Limits {
                sql_bytes: 0,
                ..Limits::default()
            },
            Limits {
                depth: 0,
                ..Limits::default()
            },
        ] {
            assert!(
                Engine::new(&catalog, &mut warm, limits)
                    .run(&key, &arguments)
                    .is_none(),
                "memoization must charge the same semantic cost"
            );
        }
    }
    assert!(observed_partial && observed_exhaustion && observed_complete);
}

#[test]
fn saved_transport_and_caller_replay_share_fail_closed_limits() {
    let files = support::prepare(&["patch-limits.mts"]);
    let path = super::tests::fixture().join("src/patch-limits.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let profile = super::tests::profile();
    let resolve =
        |specifier: &str, from: &std::path::Path| support::resolve(&files, from, specifier);
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &resolve,
    };
    let models = &files[&path].sql_identifier_syntax.recovery;
    let executors = models.executors.get(&profile).unwrap();
    let actual = |call: &&super::CallModel| {
        source[call.span.0 as usize..call.span.1 as usize].starts_with("query(statement)")
    };
    let sink = models.calls.iter().find(actual).unwrap();
    let transport = models
        .calls
        .iter()
        .find(|call| {
            source[call.span.0 as usize..call.span.1 as usize].starts_with("executeBudget(read)")
        })
        .unwrap();
    for is_transport in [true, false] {
        let mut cache = Cache::default();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let result = if is_transport {
            engine.transport(&path, transport, executors)
        } else {
            engine.site(&path, sink, executors)
        }
        .unwrap();
        assert!(result.complete());
        assert!(!result.alternatives.is_empty());
        assert!(result
            .alternatives
            .iter()
            .all(|sql| sql.sql.contains("OFFSET 9")));
        for work in [0, 1] {
            // Fresh and warm summaries must both refuse an exhausted root.
            let mut engine = Engine::new(
                &catalog,
                &mut cache,
                Limits {
                    work,
                    ..Limits::default()
                },
            );
            let result = if is_transport {
                engine.transport(&path, transport, executors)
            } else {
                engine.site(&path, sink, executors)
            };
            assert!(result.as_ref().is_none_or(|result| !result.complete()));
            assert!(engine.residual());
        }
    }
}
