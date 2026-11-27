use super::{super::*, facts};

fn roots() -> Vec<PostgresSqlExpression> {
    let facts = facts("expression-roots.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let PostgresSqlStatementKind::CreateTable { columns, .. } =
        facts.statements.into_iter().next().unwrap().facts
    else {
        panic!()
    };
    columns
        .into_iter()
        .map(|column| {
            column
                .default
                .or_else(|| column.generated.map(|generated| generated.expression))
                .unwrap()
        })
        .collect()
}

#[test]
fn direct_calls_are_distinct_from_enclosing_case_operators_and_casts() {
    use PostgresSqlExpressionRoot as Root;
    let expressions = roots();
    assert!(matches!(expressions[0].root, Root::FunctionCall { .. }));
    assert!(
        matches!(&expressions[1].root, Root::Parenthesized { expression } if matches!(expression.as_ref(), Root::Case))
    );
    assert!(matches!(expressions[2].root, Root::FunctionCall { .. }));
    assert!(matches!(&expressions[3].root, Root::Binary { operator } if operator == "+"));
    let Root::Cast {
        expression,
        data_type,
        ..
    } = &expressions[4].root
    else {
        panic!()
    };
    assert_eq!(data_type, "UUID");
    let Root::Parenthesized { expression } = expression.as_ref() else {
        panic!()
    };
    let Root::FunctionCall { name, .. } = expression.as_ref() else {
        panic!()
    };
    assert_eq!(
        name.parts
            .iter()
            .map(|part| (&part.identity, part.quoted))
            .collect::<Vec<_>>(),
        [(&"App".to_string(), true), (&"MakeId".to_string(), true)]
    );
    assert!(matches!(expressions[5].root, Root::Unary { .. }));
    assert_eq!(
        expressions[0].functions,
        expressions[1]
            .functions
            .iter()
            .cloned()
            .map(|mut function| {
                function.span = expressions[0].functions[0].span.clone();
                function
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(expressions[2].columns, expressions[3].columns);
}

#[test]
fn ordered_arguments_keep_direct_columns_separate_from_nested_mentions() {
    use PostgresSqlExpressionRoot as Root;
    let expressions = roots();
    let Root::FunctionCall {
        arguments,
        arguments_complete,
        syntax,
        modifiers,
        ..
    } = &expressions[6].root
    else {
        panic!()
    };
    assert!(*arguments_complete);
    assert_eq!(*syntax, PostgresSqlFunctionSyntax::Call);
    assert!(modifiers.is_empty());
    assert_eq!(arguments.len(), 10);
    for argument in &arguments[..4] {
        assert!(matches!(argument.root, Root::ColumnReference { .. }));
        assert!(argument.span.is_some());
    }
    let Root::ColumnReference { name } = &arguments[2].root else {
        panic!()
    };
    assert!(name.parts.iter().all(|part| part.quoted));
    assert_eq!(arguments[0].root, arguments[3].root);
    assert!(matches!(arguments[4].root, Root::Binary { .. }));
    assert!(matches!(arguments[5].root, Root::FunctionCall { .. }));
    assert!(matches!(arguments[6].root, Root::Literal { .. }));
    assert!(matches!(arguments[7].root, Root::Parenthesized { .. }));
    assert!(matches!(arguments[8].root, Root::Cast { .. }));
    assert_eq!(arguments[9].name.as_ref().unwrap().identity, "x");
}

#[test]
fn value_functions_modifiers_and_unprojected_arguments_are_explicit() {
    use PostgresSqlExpressionRoot as Root;
    let expressions = roots();
    for index in [7, 9] {
        assert!(matches!(
            expressions[index].root,
            Root::FunctionCall {
                syntax: PostgresSqlFunctionSyntax::Value,
                ..
            }
        ));
    }
    assert!(matches!(
        expressions[8].root,
        Root::FunctionCall {
            syntax: PostgresSqlFunctionSyntax::Call,
            ..
        }
    ));
    assert!(matches!(expressions[10].root, Root::Literal { .. }));
    assert!(matches!(
        expressions[11].root,
        Root::FunctionCall {
            arguments_complete: false,
            ..
        }
    ));
    for expression in &expressions[12..16] {
        assert!(
            matches!(&expression.root, Root::FunctionCall { modifiers, .. } if !modifiers.is_empty())
        );
    }
    assert!(matches!(expressions[16].root, Root::Other));
    assert!(matches!(expressions[17].root, Root::Subquery));
}

#[test]
fn parser_specific_argument_lists_and_aggregate_modifiers_remain_honest() {
    use PostgresSqlExpressionRoot as Root;
    let expressions = roots();
    assert!(matches!(
        expressions[19].root,
        Root::FunctionCall {
            arguments_complete: false,
            ..
        }
    ));
    let Root::FunctionCall {
        arguments,
        arguments_complete,
        ..
    } = &expressions[20].root
    else {
        panic!()
    };
    assert!(*arguments_complete);
    assert_eq!(arguments[0].name.as_ref().unwrap().identity, "document");
    assert!(matches!(
        expressions[21].root,
        Root::FunctionCall {
            arguments_complete: false,
            ..
        }
    ));
    for expression in &expressions[22..24] {
        assert!(
            matches!(&expression.root, Root::FunctionCall { modifiers, .. } if !modifiers.is_empty())
        );
    }
    assert!(matches!(expressions[24].root, Root::TypedLiteral { .. }));
}
