use super::*;
use crate::codebase::rules::postgres_sql_cross_file::walk_expression;

#[test]
fn assignment_identifier_summaries_do_not_duplicate_opaque_payloads() {
    let (facts, files) = prepared(&["assignment-facts.tsx"]);
    let flow = &facts.get_ts_facts(&files[0]).unwrap().value_flow;
    for name in [
        "objectValue",
        "arrayValue",
        "jsxValue",
        "fragmentValue",
        "arrowValue",
        "functionValue",
        "dynamicValue",
        "sequencePayload",
    ] {
        assert_eq!(
            assigned(flow, name).value,
            ValueExpression::Unknown,
            "{name}"
        );
    }
    for write in &flow.identifier_writes {
        if let Some(assigned) = &write.assigned {
            walk_expression(&assigned.value, &mut |value| {
                assert!(
                    !matches!(
                        value,
                        ValueExpression::Object(_, _)
                            | ValueExpression::Array(_)
                            | ValueExpression::Callable(_)
                    ),
                    "{} retained {value:?}",
                    write.name
                );
            });
        }
    }
    for name in ["projectedObject", "projectedArray"] {
        assert!(
            matches!(&assigned(flow, name).value, ValueExpression::Member(base, _) if matches!(base.as_ref(), ValueExpression::Unknown))
        );
    }
    for name in ["importedTable", "unboundTable", "destructured", "compound"] {
        assert!(
            !flow
                .identifier_writes
                .iter()
                .any(|write| write.name == name && write.assigned.is_some()),
            "{name}"
        );
    }
}

#[test]
fn assignment_call_summaries_reuse_prepared_call_spans() {
    let (facts, files) = prepared(&["assignment-facts.tsx"]);
    let flow = &facts.get_ts_facts(&files[0]).unwrap().value_flow;
    for name in ["callValue", "awaitedCallValue"] {
        let write = assigned(flow, name);
        let value = match &write.value {
            ValueExpression::Awaited(inner) => inner.as_ref(),
            value => value,
        };
        let ValueExpression::Call(start, end) = value else {
            panic!("missing call {name}")
        };
        let call = flow
            .calls
            .iter()
            .find(|call| (call.offset, call.end) == (*start, *end))
            .unwrap();
        assert_eq!(call.source_callee, "identity");
        assert_eq!(call.context, write.owner);
        assert!(
            matches!(call.arguments.as_slice(), [ValueExpression::Named { name, .. }] if name == "input")
        );
    }
}
