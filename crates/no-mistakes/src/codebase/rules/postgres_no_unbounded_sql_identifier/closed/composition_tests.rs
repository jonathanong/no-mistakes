use super::*;
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::rules::postgres_sql_cross_file::{compile, graph_plan, Options};
use crate::codebase::ts_resolver::{load_tsconfig, normalize_path};
use crate::codebase::ts_source::facts::TsFactPlan;

#[test]
fn guarded_compositions_preserve_capture_context_for_every_query() {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-guard-composition"),
    );
    for (name, count, expected) in [
        ("local-template", 5, true),
        ("alias-template", 2, true),
        ("early-return", 2, true),
        ("or-throw", 4, true),
        ("direct", 2, true),
        ("lookup", 2, true),
        ("multi-hole", 2, true),
        ("helper", 2, true),
        ("property", 2, true),
        ("captured", 2, true),
        ("negative", 16, false),
    ] {
        let config: NoMistakesConfig = serde_yaml::from_str(
            &std::fs::read_to_string(root.join(format!(".no-mistakes-{name}.yml"))).unwrap(),
        )
        .unwrap();
        let mut options: Options = config.rules[0].try_rule_options().unwrap();
        options.include.clear();
        let options = compile("postgres-no-unbounded-sql-identifier", &options).unwrap();
        let files: Vec<_> = [
            format!("{name}.ts"),
            "db.ts".into(),
            "packages/pkg/config.ts".into(),
        ]
        .iter()
        .map(|file| root.join(file))
        .collect();
        let facts = collect_check_facts(
            &root,
            files.clone(),
            CheckFactPlan {
                imports: true,
                source: true,
                embedded_sql: true,
                embedded_sql_options: vec![options.embedded.clone()],
                graph: TsFactPlan {
                    value_flow: true,
                    identifier_provenance: true,
                    function_calls: true,
                    source: true,
                    ..TsFactPlan::imports()
                },
                ..Default::default()
            },
        );
        let graph = DepGraph::build_with_plan_file_list_and_check_facts(
            &root,
            &load_tsconfig(&root.join("tsconfig.json")).unwrap(),
            graph_plan(&config).unwrap(),
            files,
            &facts,
        )
        .unwrap();
        let calls = sql_calls(&root, &options, &facts);
        assert_eq!(calls.len(), count, "all first and later queries: {name}");
        let bindings = Bindings::with_calls(&facts, &calls)
            .with_graph(&graph)
            .with_numeric_parameter_bounds();
        for call in &calls {
            let sources = identifier_sources(&graph, &facts, call, &bindings);
            assert_eq!(sources.len(), 1, "{name}: {:?}", call.span);
            for source in sources {
                assert_eq!(
                    super::super::proven(&graph, &options, &facts, &bindings, call, &source),
                    expected,
                    "{name}: {:?} {source:?}",
                    call.span
                );
            }
        }
        let flow = &facts
            .get_ts_facts(&root.join(format!("{name}.ts")))
            .unwrap()
            .value_flow;
        if let Some(origins) = &flow.identifier_text_origins {
            for binding in flow.bindings.iter().filter(|binding| {
                matches!(
                    binding.value,
                    ValueExpression::Object(_, _) | ValueExpression::Array(_)
                )
            }) {
                assert!(origins
                    .binding(binding.scope, &binding.name, binding.declared_at)
                    .is_none());
            }
        }
    }
}
