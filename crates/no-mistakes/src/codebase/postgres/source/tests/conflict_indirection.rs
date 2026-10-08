use super::{facts, fixture};
use crate::codebase::postgres::source::*;

fn slice<'a>(sql: &'a str, span: &Option<PostgresSqlSpan>) -> &'a str {
    let span = span.as_ref().expect("source span");
    &sql[span.start.offset..span.end.offset]
}

#[test]
fn conflict_indirection_operator_classes_and_nested_function_spans_are_complete() {
    let sql = fixture("insert-conflict-indirection.sql");
    let result = facts("insert-conflict-indirection.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 5);
    let inserts = result.statements[..4]
        .iter()
        .map(|statement| {
            let PostgresSqlStatementKind::Insert { insert } = &statement.facts else {
                panic!("insert")
            };
            insert
        })
        .collect::<Vec<_>>();
    let PostgresSqlConflictTarget::Expressions {
        expressions,
        operator_classes,
    } = &inserts[0].on_conflict.as_ref().unwrap().target
    else {
        panic!("expressions")
    };
    assert!(operator_classes.is_none());
    assert_eq!(
        slice(&sql, &expressions[0].functions[0].span),
        "lower(upper(slug))"
    );
    assert_eq!(
        slice(&sql, &expressions[0].functions[1].span),
        "upper(slug)"
    );
    let PostgresSqlConflictAction::DoUpdate { assignments, .. } =
        &inserts[0].on_conflict.as_ref().unwrap().action
    else {
        panic!("update")
    };
    let PostgresSqlExpressionRoot::FunctionCall {
        arguments,
        arguments_complete,
        ..
    } = &assignments[2].expression.root
    else {
        panic!("wildcard call")
    };
    assert!(!arguments_complete);
    assert!(arguments[0].span.is_none());
    // Unary wrappers must traverse the same prepared nested call spans.
    let unary = &assignments[3];
    assert!(unary.complete);
    assert_eq!(
        slice(&sql, &unary.expression.span),
        "-abs(length(lower(slug)))"
    );
    let PostgresSqlExpressionRoot::Unary { expression, .. } = &unary.expression.root else {
        panic!("unary")
    };
    let PostgresSqlExpressionRoot::FunctionCall { arguments, .. } = expression.as_ref() else {
        panic!("unary function")
    };
    assert_eq!(slice(&sql, &arguments[0].span), "length(lower(slug))");
    let PostgresSqlExpressionRoot::FunctionCall { arguments, .. } = &arguments[0].root else {
        panic!("unary nested function")
    };
    assert_eq!(slice(&sql, &arguments[0].span), "lower(slug)");
    let expression = &assignments[0].expression;
    let PostgresSqlExpressionRoot::FunctionCall { arguments, .. } = &expression.root else {
        panic!("function")
    };
    assert_eq!(slice(&sql, &arguments[1].span), "lower(upper(slug))");
    let PostgresSqlExpressionRoot::FunctionCall { arguments, .. } = &arguments[1].root else {
        panic!("nested function")
    };
    assert_eq!(slice(&sql, &arguments[0].span), "upper(slug)");
    assert_eq!(
        slice(&sql, &expression.functions[0].span),
        "coalesce(EXCLUDED.slug, lower(upper(slug)))"
    );
    let PostgresSqlConflictAction::DoUpdate { assignments, .. } =
        &inserts[1].on_conflict.as_ref().unwrap().action
    else {
        panic!("update")
    };
    for (index, count) in [(0, 2), (1, 4)] {
        let target = assignments[index].target.as_ref().unwrap();
        assert_eq!(target.indirection.as_ref().unwrap().len(), count);
        assert_eq!(target.base.sql, "records");
        assert!(slice(&sql, &target.span).ends_with(".name"));
    }
    // A composite field needs a target even without any array subscripts.
    let field_only = assignments[3].target.as_ref().unwrap();
    assert_eq!(field_only.base.sql, "records");
    assert_eq!(assignments[3].columns[0].sql, "records");
    assert!(assignments[3].complete);
    assert!(field_only.subscripts.is_empty());
    assert_eq!(slice(&sql, &field_only.span), "records.\"Name\"");
    let PostgresSqlAssignmentStep::Field { name, span } =
        &field_only.indirection.as_ref().unwrap()[0]
    else {
        panic!("field-only step")
    };
    assert!(name.quoted);
    assert_eq!(slice(&sql, span), "\"Name\"");
    let leading = assignments[2].target.as_ref().unwrap();
    assert_eq!(leading.base.sql, "records");
    assert_eq!(assignments[2].columns[0].sql, "records");
    assert_eq!(leading.indirection.as_ref().unwrap().len(), 5);
    let PostgresSqlAssignmentStep::Field { name, span } = &leading.indirection.as_ref().unwrap()[0]
    else {
        panic!("leading field")
    };
    assert!(name.quoted);
    assert_eq!(slice(&sql, span), "\"Items\"");
    let steps = assignments[1]
        .target
        .as_ref()
        .unwrap()
        .indirection
        .as_ref()
        .unwrap();
    let PostgresSqlAssignmentStep::Field { name, span } = &steps[1] else {
        panic!("field")
    };
    assert!(name.quoted);
    assert_eq!(slice(&sql, span), "\"Items\"");
    let PostgresSqlAssignmentStep::Subscript { expression, span } = &steps[2] else {
        panic!("index")
    };
    assert_eq!(slice(&sql, span), "[coalesce(records[1].idx, 2)]");
    assert_eq!(
        slice(&sql, &expression.functions[0].span),
        "coalesce(records[1].idx, 2)"
    );
    for (index, expected) in [
        (2, vec!["text_pattern_ops", "\"Ops\".\"IntOps\""]),
        (3, vec!["\"Ops\".\"TextOps\""]),
    ] {
        let PostgresSqlConflictTarget::Expressions {
            expressions,
            operator_classes,
        } = &inserts[index].on_conflict.as_ref().unwrap().target
        else {
            panic!("operator class")
        };
        let classes = operator_classes.as_ref().unwrap();
        assert_eq!(expressions.len(), classes.len());
        if index == 2 {
            assert!(classes[2].is_none());
        }
        for (class, expected) in classes.iter().zip(expected) {
            let class = class.as_ref().unwrap();
            let expected_span = if !class.parameters.is_empty() {
                format!("{expected} (siglen = 32)")
            } else {
                expected.to_string()
            };
            assert_eq!(slice(&sql, &class.span), expected_span);
            assert_eq!(class.name.sql, expected);
        }
    }
}

#[test]
fn conflict_parameters_and_predicates_reuse_exact_prepared_spans() {
    let sql = fixture("insert-conflict-indirection.sql");
    let result = facts("insert-conflict-indirection.sql");
    let PostgresSqlStatementKind::Insert { insert } = &result.statements[2].facts else {
        panic!("insert")
    };
    let conflict = insert.on_conflict.as_ref().unwrap();
    let PostgresSqlConflictAction::DoUpdate { predicate, .. } = &conflict.action else {
        panic!("update")
    };
    for (predicate, expected, nested) in [
        (
            conflict.predicate.as_ref().unwrap(),
            "coalesce(is_ready(lower(slug)), false)",
            "is_ready(lower(slug))",
        ),
        (
            predicate.as_ref().unwrap().as_ref(),
            "coalesce(is_ready(upper(slug)), true)",
            "is_ready(upper(slug))",
        ),
    ] {
        assert_eq!(slice(&sql, &predicate.span), expected);
        assert_eq!(slice(&sql, &predicate.functions[0].span), expected);
        assert_eq!(slice(&sql, &predicate.functions[1].span), nested);
        let PostgresSqlExpressionRoot::FunctionCall { arguments, .. } = &predicate.root else {
            panic!("predicate call")
        };
        assert_eq!(slice(&sql, &arguments[0].span), nested);
        let PostgresSqlExpressionRoot::FunctionCall { arguments, .. } = &arguments[0].root else {
            panic!("nested predicate call")
        };
        assert_eq!(
            slice(&sql, &arguments[0].span),
            if expected.ends_with("false)") {
                "lower(slug)"
            } else {
                "upper(slug)"
            }
        );
    }
    let PostgresSqlConflictTarget::Expressions {
        operator_classes: Some(classes),
        ..
    } = &conflict.target
    else {
        panic!("classes")
    };
    let parameter = &classes[0].as_ref().unwrap().parameters[0];
    assert_eq!(parameter.name.identity, "siglen");
    assert_eq!(slice(&sql, &parameter.value.span), "32");
}
