use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::rules::postgres_sql_cross_file::graph_plan;
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn imported_record_fields_and_nested_catalog_pipelines_require_closed_uses() {
    let root = normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-catalogs",
    ));
    let files: Vec<_> = [
        "catalog.ts",
        "barrel.ts",
        "hierarchy.ts",
        "providers.ts",
        "provider-barrel.ts",
        "lookup.ts",
        "lookup-barrel.ts",
        "collections.ts",
        "open.ts",
        "mutated.ts",
        "escaped.ts",
        "collections-unsafe.ts",
        "shared-unsafe-provider.ts",
        "shared-unsafe-barrel.ts",
        "shared-unsafe-consumer.ts",
        "namespace-member-provider.ts",
        "namespace-member-consumer.ts",
        "namespace-object-provider.ts",
        "namespace-object-consumer.ts",
    ]
    .iter()
    .map(|file| root.join("src").join(file))
    .collect();
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-catalogs/.no-mistakes.yml"
    ))).unwrap();
    let plan = graph_plan(&config).unwrap();
    assert!(plan.calls && plan.imports);
    let fact_plan = TsFactPlan {
        value_flow: true,
        source: true,
        ..TsFactPlan::imports()
    };
    let prepared = collect_ts_facts(&files, fact_plan);
    let mut facts = CheckFactMap {
        files: files.clone(),
        graph_plan: fact_plan,
        ..Default::default()
    };
    for file in &files {
        let ts = prepared.get(file).unwrap();
        assert!(ts.parse_error.is_none(), "{}", file.display());
        facts.ts.insert(
            file.clone(),
            CheckFileFacts {
                ts: Arc::new(ts.clone()),
                ..Default::default()
            }
            .into(),
        );
    }
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph =
        DepGraph::build_with_plan_file_list_and_check_facts(&root, &tsconfig, plan, files, &facts)
            .unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let proof = CatalogProof::new(&graph, &facts, &bindings);
    for (file, function, name, members, expected) in [
        (
            "namespace-member-consumer.ts",
            "namespaceMemberEscape",
            "config",
            vec!["emailColumn"],
            false,
        ),
        (
            "namespace-object-consumer.ts",
            "namespaceObjectEscape",
            "config",
            vec!["emailColumn"],
            false,
        ),
        (
            "lookup.ts",
            "getAccount",
            "config",
            vec!["accountIdColumn"],
            true,
        ),
        (
            "lookup.ts",
            "getAccount",
            "config",
            vec!["providerUserIdColumn"],
            true,
        ),
        (
            "lookup.ts",
            "getAccount",
            "config",
            vec!["emailColumn"],
            true,
        ),
        (
            "lookup.ts",
            "getAccount",
            "config",
            vec!["unknownColumn"],
            false,
        ),
        (
            "lookup-barrel.ts",
            "barrelLookup",
            "config",
            vec!["emailColumn"],
            true,
        ),
        ("lookup-barrel.ts", "barrelLookup", "email", vec![], true),
        (
            "open.ts",
            "openProvider",
            "config",
            vec!["emailColumn"],
            false,
        ),
        (
            "mutated.ts",
            "mutatedProvider",
            "config",
            vec!["emailColumn"],
            false,
        ),
        (
            "mutated.ts",
            "aliasedProvider",
            "config",
            vec!["emailColumn"],
            false,
        ),
        (
            "mutated.ts",
            "rawProvider",
            "config",
            vec!["emailColumn"],
            false,
        ),
        (
            "escaped.ts",
            "escapedProvider",
            "config",
            vec!["emailColumn"],
            false,
        ),
        (
            "escaped.ts",
            "exportedProvider",
            "config",
            vec!["emailColumn"],
            false,
        ),
        (
            "escaped.ts",
            "getterProvider",
            "config",
            vec!["emailColumn"],
            false,
        ),
        (
            "shared-unsafe-consumer.ts",
            "importedEscape",
            "config",
            vec!["emailColumn"],
            false,
        ),
        (
            "shared-unsafe-consumer.ts",
            "sharedSource",
            "config",
            vec!["emailColumn"],
            false,
        ),
    ] {
        let file = root.join("src").join(file);
        let ts = facts.get_ts_facts(&file).unwrap();
        let owner = ts
            .callable_scope_ids
            .iter()
            .find(|(_, name)| name == function)
            .unwrap()
            .0;
        let binding = ts
            .value_flow
            .bindings
            .iter()
            .find(|binding| binding.owner == Some(owner) && binding.name == name)
            .unwrap();
        let members: Vec<_> = members.into_iter().map(String::from).collect();
        assert_eq!(
            proof.scalar(&file, &binding.value, &members, &mut FxHashSet::default()),
            expected,
            "{function} {name} {members:?}"
        );
    }
    for (file, name, expected) in [
        ("collections.ts", "SELECT_SQL", true),
        ("collections-unsafe.ts", "MUTATING_SQL", false),
        ("collections-unsafe.ts", "FILTER_SQL", false),
        ("collections-unsafe.ts", "OPEN_SQL", false),
    ] {
        let file = root.join("src").join(file);
        let binding = facts
            .get_ts_facts(&file)
            .unwrap()
            .value_flow
            .bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap();
        assert_eq!(proof.pipeline(&file, &binding.value), expected, "{name}");
    }
    let provider = root.join("src/providers.ts");
    let catalog = &facts
        .get_ts_facts(&provider)
        .unwrap()
        .value_flow
        .catalogs
        .sources[0];
    assert_eq!(catalog.fields.len(), 3);
    assert!(catalog
        .paths
        .contains(&vec!["github".into(), "emailColumn".into()]));
    assert!(proof
        .safe_sources
        .contains(&(provider.as_path(), 0, "IDENTIFIERS")));
}
