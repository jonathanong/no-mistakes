use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::GuardKind;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::rules::postgres_sql_cross_file::{compile, graph_plan, Options};
use crate::codebase::ts_resolver::{normalize_path, TsConfig};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use crate::config::v2::{EffectiveRuleAnalysis, Effort, NoMistakesConfig};
use std::path::PathBuf;
use std::sync::Arc;

fn config() -> NoMistakesConfig {
    serde_yaml::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards/.no-mistakes.yml"))).unwrap()
}

#[test]
fn configured_module_validators_demand_and_match_canonical_reexport_targets() {
    let root = normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-guards",
    ));
    let files: Vec<_> = [
        "guarded.ts",
        "unguarded.ts",
        "getters.ts",
        "barrel.ts",
        "validators.ts",
        "other.ts",
    ]
    .iter()
    .map(|name| root.join("src").join(name))
    .collect();
    let config = config();
    let plan = graph_plan(&config).unwrap();
    assert!(plan.calls && plan.imports && plan.effect_selectors);
    let options: Options = config.rules[0].try_rule_options().unwrap();
    let compiled = compile("postgres-no-unbounded-sql-identifier", &options).unwrap();
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
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &tsconfig,
        plan,
        files.clone(),
        &facts,
    )
    .unwrap();
    assert_eq!(
        graph
            .module_export_nodes("./validators", "assertIdentifier")
            .len(),
        1
    );
    for (file, function, callee, expected) in [
        ("guarded.ts", "envAlias", "validate", true),
        ("guarded.ts", "property", "validate", true),
        ("guarded.ts", "property", "validateRemote", true),
        (
            "guarded.ts",
            "namespace",
            "validators.validateIdentifier",
            true,
        ),
        ("guarded.ts", "localBinding", "validate", true),
        ("unguarded.ts", "wrongModule", "assertIdentifier", false),
        ("unguarded.ts", "shadowedModule", "validate", false),
    ] {
        let file = root.join("src").join(file);
        let ts = facts.get_ts_facts(&file).unwrap();
        let owner = ts
            .callable_scope_ids
            .iter()
            .find(|(_, name)| name == function)
            .unwrap()
            .0;
        let guard = ts.value_flow.finite_identifier_guards.iter().find(|guard| {
            guard.context == Some(owner) && matches!(&guard.kind, GuardKind::Assertion { callee: name, argument: 0, .. } if name == callee)
        }).unwrap();
        let GuardKind::Assertion { call, .. } = &guard.kind else {
            unreachable!();
        };
        assert_eq!(
            matches(&graph, &compiled, &file, callee, 0, *call),
            expected,
            "{function}"
        );
    }
    for plan in [
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
        GraphBuildPlan {
            imports: true,
            ..Default::default()
        },
    ] {
        let graph = DepGraph::build_with_plan_file_list_and_check_facts(
            &root,
            &tsconfig,
            plan,
            files.clone(),
            &facts,
        )
        .unwrap();
        assert!(graph
            .module_export_nodes("./validators", "assertIdentifier")
            .is_empty());
    }
}

#[test]
fn disabled_low_and_name_only_validator_options_do_not_demand_module_catalogs() {
    let mut disabled = config();
    disabled.rules[0].enabled = false;
    assert!(graph_plan(&disabled).is_none());
    let mut names = config();
    names.rules[0]
        .options
        .get_mut("identifierValidators")
        .unwrap()
        .as_sequence_mut()
        .unwrap()
        .retain(|validator| validator.get("module").is_none());
    assert!(!graph_plan(&names).unwrap().effect_selectors);
    let mut mixed = config();
    mixed.rules[0].resolved_analysis = Some(EffectiveRuleAnalysis {
        effort: Effort::Low,
        ..Default::default()
    });
    mixed.rules.push(crate::config::v2::schema::RuleDef {
        rule: "postgres-no-mutable-sql-registry".into(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        ..Default::default()
    });
    assert!(!graph_plan(&mixed).unwrap().effect_selectors);
}
