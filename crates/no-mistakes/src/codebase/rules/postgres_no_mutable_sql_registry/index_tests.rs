use super::super::postgres_sql_cross_file::{registry_route, RegistryRoutes};
use super::write_index::WriteIndex;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId, TsFactLookup};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn many_require_routes_index_fact_rows_and_edges_once() {
    let root = crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-mutable-sql-registry/fixture/fail-many-const-requires",
    ));
    let writer = root.join("src/mutate.ts");
    let producer = root.join("src/tables.ts");
    let files = vec![writer.clone(), producer.clone()];
    let collected = collect_ts_facts(
        &files,
        TsFactPlan {
            imports: true,
            function_calls: true,
            value_flow: true,
            ..Default::default()
        },
    );
    let mut facts = CheckFactMap {
        files,
        ..Default::default()
    };
    for (file, ts) in collected {
        facts.ts.insert(
            file,
            CheckFileFacts {
                ts: Arc::new(ts),
                ..Default::default()
            }
            .into(),
        );
    }
    let mut forward = crate::fx::fx_map();
    forward.insert(
        NodeId::file(&writer),
        vec![(NodeId::file(&producer), EdgeKind::Require)],
    );
    let graph = from_typed_maps(root, forward, Default::default());
    let routes = RegistryRoutes::new(&facts);
    // Cold, unused route indexes do not visit or retain any fact rows.
    assert_eq!(routes.work.get(), [0; 3]);
    let ts = facts.get_ts_facts(&writer).unwrap();
    let writes = WriteIndex::new(Some(ts));
    let bindings = &ts.value_flow.bindings;
    assert_eq!(bindings.len(), 96);
    let mut first = None;
    for _ in 0..2 {
        for binding in bindings {
            assert!(writes.member_write(&binding.name));
            let members = if matches!(binding.value, ValueExpression::Member(_, _)) {
                Vec::new()
            } else {
                vec!["tables".into()]
            };
            let result = registry_route(&graph, &routes, &writer, &binding.name, &members)
                .unwrap_or_else(|| {
                    panic!(
                        "missing {} {:?}; work {:?}",
                        binding.name,
                        binding.value,
                        routes.work.get()
                    )
                });
            assert_eq!(result.producer, producer);
            assert_eq!(result.exported, "tables");
        }
        let work = routes.work.get();
        assert_eq!(work[0], 2, "each demanded file is indexed once");
        assert_eq!(
            work[2], 1,
            "the require edge projection is shared across bindings"
        );
        assert!(
            work[1] <= 5 * 96 + 8,
            "actual fact/edge row visits: {work:?}"
        );
        assert!(
            writes.work > 0 && writes.work <= 8 * 96,
            "writer row visits: {}",
            writes.work
        );
        if let Some(prior) = first {
            assert_eq!(work, prior, "lookups must not rebuild indexes");
        }
        first = Some(work);
    }
}

#[test]
fn ordinary_esm_lookup_does_not_build_require_or_export_indexes() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-mutable-sql-registry/fixture/fail-many-const-requires"),
    );
    let file = root.join("src/query.ts");
    let files = vec![file.clone()];
    let collected = collect_ts_facts(
        &files,
        TsFactPlan {
            imports: true,
            function_calls: true,
            value_flow: true,
            ..Default::default()
        },
    );
    let mut facts = CheckFactMap {
        files,
        ..Default::default()
    };
    for (path, ts) in collected {
        facts.ts.insert(
            path,
            CheckFileFacts {
                ts: Arc::new(ts),
                ..Default::default()
            }
            .into(),
        );
    }
    let graph = from_typed_maps(root, Default::default(), Default::default());
    let routes = RegistryRoutes::new(&facts);
    assert_eq!(routes.work.get(), [0; 3]);
    for _ in 0..2 {
        assert!(registry_route(&graph, &routes, &file, "notARequire", &[]).is_none());
        // Only the two import rows are demanded: the raw query call is not read.
        assert_eq!(routes.work.get(), [1, 2, 0]);
    }
}
