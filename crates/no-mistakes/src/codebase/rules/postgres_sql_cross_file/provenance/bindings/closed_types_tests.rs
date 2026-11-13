use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{CallableId, ValueBinding};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::ts_resolver::{load_tsconfig, normalize_path};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use crate::fx::fx_set;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn identifier_parameter_bounds_require_runtime_proof_for_catalog_types() {
    let file = PathBuf::from("query.ts");
    let mut ts = TsFileFacts::default();
    ts.value_flow.bindings.push(binding(
        "TABLES",
        0,
        ValueExpression::Array(Arc::new(Default::default())),
    ));
    ts.value_flow
        .type_shapes
        .insert("Allowed".into(), ValueType::Element("TABLES".into()));
    for (index, shape) in [
        Some(ValueType::Element("TABLES".into())),
        Some(ValueType::Reference("Allowed".into())),
        Some(ValueType::Number),
        None,
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("parameter{index}");
        ts.value_flow.parameters.push(parameter(&name, shape));
        ts.value_flow.bindings.push(binding(
            &format!("alias{index}"),
            1,
            ValueExpression::Named {
                name,
                scope: Some(1),
                offset: 10,
                context: Some(CallableId(1)),
            },
        ));
    }
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
    let shared = Bindings::new(&facts);
    let identifier = Bindings::new(&facts).with_numeric_parameter_bounds();
    for index in 0..4 {
        let name = format!("parameter{index}");
        assert_eq!(shared.closed_parameter(&file, 1, &name), index != 3);
        assert_eq!(identifier.closed_parameter(&file, 1, &name), index == 2);
        let alias = format!("alias{index}");
        assert_eq!(shared.flows_unbounded(&file, 1, &alias, 20), index == 3);
        assert_eq!(identifier.flows_unbounded(&file, 1, &alias, 20), index != 2);
    }
}

#[test]
fn saved_cross_file_catalog_types_require_a_unique_literal_export() {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("3304-crossfile-closed-types"),
    );
    let files: Vec<_> = ["src/catalog.ts", "src/query.ts"]
        .iter()
        .map(|file| root.join(file))
        .collect();
    let plan = TsFactPlan {
        value_flow: true,
        ..TsFactPlan::imports()
    };
    let facts = CheckFactMap {
        files: files.clone(),
        graph_plan: plan,
        ts: collect_ts_facts(&files, plan)
            .into_iter()
            .map(|(file, ts)| {
                assert!(ts.parse_error.is_none(), "{file:?}: {:?}", ts.parse_error);
                (
                    file,
                    CheckFileFacts {
                        ts: Arc::new(ts),
                        ..Default::default()
                    }
                    .into(),
                )
            })
            .collect(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &load_tsconfig(&root.join("tsconfig.json")).unwrap(),
        GraphBuildPlan {
            imports: true,
            value_import_selectors: true,
            ..Default::default()
        },
        files,
        &facts,
    )
    .unwrap();
    let query = root.join("src/query.ts");
    let query_facts = facts.get_ts_facts(&query).unwrap();
    let bindings = Bindings::new(&facts).with_graph(&graph);
    for (name, expected) in [
        ("importedType", true),
        ("importedCatalogElement", true),
        ("localCatalogAlias", true),
        ("numeric", true),
        ("openString", false),
    ] {
        let parameter = query_facts
            .value_flow
            .parameters
            .iter()
            .find(|parameter| parameter.name == name)
            .unwrap_or_else(|| panic!("missing parameter {name}"));
        assert_eq!(bindings.closed_type(&query, parameter), expected, "{name}");
    }
}

#[test]
fn closed_types_reject_missing_facts_unresolved_aliases_and_uncatalogued_elements() {
    let absent = PathBuf::from("absent.ts");
    let present = PathBuf::from("query.ts");
    let mut facts = CheckFactMap::default();
    facts.files.push(present.clone());
    facts.ts.insert(
        present.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts::default()),
            ..Default::default()
        }
        .into(),
    );
    let bindings = Bindings::new(&facts);
    let reference = parameter("alias", Some(ValueType::Reference("Row".into())));
    let element = parameter("item", Some(ValueType::Element("TABLES".into())));
    assert!(!bindings.closed_type(&absent, &reference));
    assert!(!bindings.closed_type(&absent, &element));
    assert!(!bindings.closed_type(&present, &reference));
    assert!(!bindings.closed_type(&present, &element));
}

fn binding(name: &str, scope: usize, value: ValueExpression) -> ValueBinding {
    ValueBinding {
        immutable: true,
        owner: (scope != 0).then_some(CallableId(1)),
        name: name.into(),
        scope,
        declared_at: 1,
        repeated: false,
        invalidated_at: None,
        mutation_contexts: fx_set(),
        value,
    }
}

fn parameter(name: &str, type_reference: Option<ValueType>) -> ValueParameter {
    ValueParameter {
        owner: CallableId(1),
        index: 0,
        name: name.into(),
        scope: 1,
        default: ValueExpression::Undefined,
        projection: Vec::new(),
        initialization_order: 0,
        default_order: 0,
        default_mutations: None,
        invalidated_at: None,
        mutation_contexts: fx_set(),
        type_reference,
    }
}
