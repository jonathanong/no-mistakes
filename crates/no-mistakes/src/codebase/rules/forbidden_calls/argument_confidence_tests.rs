use super::*;
use serde_json::json;

#[test]
fn prepared_failure_entries_never_prove_a_predicate() {
    use crate::codebase::ts_source::{
        facts::TsFileFacts,
        module_facts::{ModuleDiagnostic, TypeScriptModuleFacts},
    };
    use std::sync::Arc;
    let module = TypeScriptModuleFacts {
        call_arguments: Some(Arc::new(
            [(
                7,
                ArgumentValues {
                    values: vec![ArgumentValue::Scalar(json!(1))],
                    spread_from: None,
                    path: None,
                },
            )]
            .into_iter()
            .collect(),
        )),
        ..Default::default()
    };
    let good = TsFileFacts {
        module_bindings: Some(Arc::new(module)),
        ..Default::default()
    };
    assert!(for_call(&good, 7).is_some());
    assert!(for_call(&good, 8).is_none());
    assert!(for_call(&TsFileFacts::default(), 7).is_none());
    for kind in 0..4 {
        let mut facts = good.clone();
        match kind {
            0 => facts.operational_error = Some("saved read failure".into()),
            1 => facts.parse_error = Some("saved syntax failure".into()),
            2 => facts.fatal_parse_error = true,
            _ => Arc::make_mut(facts.module_bindings.as_mut().unwrap())
                .diagnostics
                .push(ModuleDiagnostic {
                    kind: "semanticError".into(),
                    message: "duplicate binding".into(),
                    span: None,
                }),
        }
        assert!(for_call(&facts, 7).is_none());
    }
}
