use super::*;
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::rules::postgres_sql_cross_file::{
    compile, graph_plan, identifier_sources, sql_calls, Options,
};
use crate::codebase::ts_resolver::{load_tsconfig, normalize_path};
use crate::codebase::ts_source::facts::TsFactPlan;

#[test]
fn aliased_allow_list_guards_a_transaction_after_an_early_return() {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-aliased-guards"),
    );
    let files: Vec<_> = ["save.ts", "unchecked.ts", "db.ts", "packages/pkg/config.ts"]
        .iter()
        .map(|file| root.join(file))
        .collect();
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-aliased-guards/.no-mistakes.yml"
    ))).unwrap();
    let mut options: Options = config.rules[0].try_rule_options().unwrap();
    options.include.clear();
    let options = compile("postgres-no-unbounded-sql-identifier", &options).unwrap();
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
                function_calls: true,
                source: true,
                ..TsFactPlan::imports()
            },
            ..Default::default()
        },
    );
    for file in &files {
        let ts = facts.get_ts_facts(file).unwrap();
        assert!(
            ts.parse_error.is_none(),
            "{}: {:?}",
            file.display(),
            ts.parse_error
        );
    }
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &load_tsconfig(&root.join("tsconfig.json")).unwrap(),
        graph_plan(&config).unwrap(),
        files,
        &facts,
    )
    .unwrap();
    let calls = sql_calls(&root, &options, &facts);
    assert_eq!(
        calls.len(),
        2,
        "both guarded and unchecked transaction SQL must be analyzed"
    );
    let bindings = Bindings::with_calls(&facts, &calls).with_graph(&graph);
    let export = unique_export(
        &graph,
        &facts,
        &bindings.routes,
        &root.join("save.ts"),
        "ALLOWED_TABLES",
    )
    .unwrap();
    assert_eq!(export.producer, root.join("packages/pkg/config.ts"));
    assert_eq!(export.exported, "ALLOWED_TABLES");
    assert!(proven(
        &graph,
        &facts,
        &bindings,
        &root.join("save.ts"),
        "ALLOWED_TABLES",
        RegistryKind::Array
    ));
    for call in &calls {
        let guarded = call.file.ends_with("save.ts");
        let sources = identifier_sources(&graph, &facts, call, &bindings);
        assert_eq!(sources.len(), 1, "{}: {sources:?}", call.file.display());
        let source = sources.iter().next().unwrap();
        assert!(
            matches!(source, Provenance::Reference { name, members, .. } if name == "tableName" && members.is_empty())
        );
        assert_eq!(
            super::super::proven(&graph, &options, &facts, &bindings, call, source),
            guarded,
            "{}",
            call.file.display()
        );
        let flow = &facts.get_ts_facts(&call.file).unwrap().value_flow;
        if guarded {
            let query = flow
                .calls
                .iter()
                .find(|value| (value.offset, value.end) == call.span)
                .unwrap();
            let guard = flow
                .finite_identifier_guards
                .iter()
                .find(|guard| {
                    guard.name == "tableName" && matches!(guard.kind, GuardKind::ClosedSet { .. })
                })
                .unwrap();
            assert_eq!(guard.context, query.context);
            assert!(guard.from < call.span.0 && call.span.1 < guard.until);
            assert!(matches!(
                guard.subject,
                Some(GuardSubject::Parameter { .. })
            ));
        } else {
            assert!(flow
                .finite_identifier_guards
                .iter()
                .all(|guard| guard.name != "tableName"));
        }
    }
}
