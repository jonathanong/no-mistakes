use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{CallableId, ValueBinding};
use crate::codebase::ts_source::facts::TsFileFacts;
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
