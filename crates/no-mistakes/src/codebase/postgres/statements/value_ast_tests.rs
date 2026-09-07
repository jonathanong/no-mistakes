use super::{extract_sql_statement_facts, SqlValueForm};
use sqlparser::ast::{
    DollarQuotedString, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArgumentList,
    FunctionArguments, Ident, ObjectName, ObjectNamePart, Query, SetExpr, UnaryOperator, Value,
    Values,
};

fn empty_query() -> Query {
    Query {
        with: None,
        body: Box::new(SetExpr::Values(Values {
            explicit_row: false,
            rows: Vec::new(),
            value_keyword: false,
        })),
        order_by: None,
        limit_clause: None,
        fetch: None,
        locks: Vec::new(),
        for_clause: None,
        settings: None,
        format_clause: None,
        pipe_operators: Vec::new(),
    }
}

fn function_expr(name: ObjectName, args: FunctionArguments) -> Expr {
    Expr::Function(Function {
        name,
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args,
        filter: None,
        null_treatment: None,
        over: None,
        within_group: Vec::new(),
    })
}

#[test]
fn value_forms_cover_named_args_and_function_name_parts() {
    let named = function_expr(
        ObjectName(vec![ObjectNamePart::Identifier(Ident::new("coalesce"))]),
        FunctionArguments::List(FunctionArgumentList {
            duplicate_treatment: None,
            args: vec![FunctionArg::Named {
                name: Ident::new("a"),
                arg: FunctionArgExpr::Expr(Expr::Identifier(Ident::new("note"))),
                operator: sqlparser::ast::FunctionArgOperator::Equals,
            }],
            clauses: Vec::new(),
        }),
    );
    assert_eq!(
        super::value::from_expr(&named),
        SqlValueForm::Coalesce {
            args: vec![SqlValueForm::SelfRef {
                column: "note".into()
            }]
        }
    );
    let expr_named = function_expr(
        ObjectName(vec![ObjectNamePart::Identifier(Ident::new("coalesce"))]),
        FunctionArguments::List(FunctionArgumentList {
            duplicate_treatment: None,
            args: vec![FunctionArg::ExprNamed {
                name: Expr::Identifier(Ident::new("a")),
                arg: FunctionArgExpr::Expr(Expr::Identifier(Ident::new("note"))),
                operator: sqlparser::ast::FunctionArgOperator::Equals,
            }],
            clauses: Vec::new(),
        }),
    );
    assert_eq!(
        super::value::from_expr(&expr_named),
        SqlValueForm::Coalesce {
            args: vec![SqlValueForm::SelfRef {
                column: "note".into()
            }]
        }
    );
    let none_args = function_expr(
        ObjectName(vec![ObjectNamePart::Identifier(Ident::new("current_user"))]),
        FunctionArguments::None,
    );
    assert_eq!(super::value::from_expr(&none_args), SqlValueForm::Other);
    let part = function_expr(
        ObjectName(vec![ObjectNamePart::Function(
            sqlparser::ast::ObjectNamePartFunction {
                name: Ident::new("fn"),
                args: Vec::new(),
            },
        )]),
        FunctionArguments::Subquery(Box::new(empty_query())),
    );
    assert_eq!(super::value::from_expr(&part), SqlValueForm::Other);
    let wildcard = function_expr(
        ObjectName(vec![ObjectNamePart::Identifier(Ident::new("count"))]),
        FunctionArguments::List(FunctionArgumentList {
            duplicate_treatment: None,
            args: vec![FunctionArg::Unnamed(FunctionArgExpr::Wildcard)],
            clauses: Vec::new(),
        }),
    );
    assert_eq!(super::value::from_expr(&wildcard), SqlValueForm::Other);
}

#[test]
fn unary_not_exists_and_unrestricted_exists_leaves() {
    let guarded = Expr::UnaryOp {
        op: UnaryOperator::Not,
        expr: Box::new(Expr::Exists {
            subquery: Box::new(empty_query()),
            negated: false,
        }),
    };
    assert!(super::not_exists::has_conjunctive_not_exists(&guarded));
    let facts = extract_sql_statement_facts(
        "SELECT 1 WHERE EXISTS (
            SELECT 1 FROM accounts WHERE id IS NULL
            UNION ALL
            SELECT 1 FROM accounts WHERE id IS NULL
         )",
    );
    assert!(
        facts.selects.iter().any(|select| select
            .exists_set_operations
            .iter()
            .any(|exists| !exists.restricted)),
        "{:#?}",
        facts.selects
    );
    let mismatched = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE items.note IS NULL AND EXCLUDED.note = 1;",
    );
    assert!(mismatched.inserts[0]
        .on_conflict
        .as_ref()
        .unwrap()
        .where_proof
        .null_and_excluded_not_null
        .is_empty());
}

#[test]
fn insert_value_stability_is_literals_nulls_and_placeholders() {
    use super::form_is_stable;
    assert!(form_is_stable(&SqlValueForm::Literal));
    assert!(form_is_stable(&SqlValueForm::Null));
    assert!(form_is_stable(&SqlValueForm::Placeholder));
    assert!(form_is_stable(&SqlValueForm::Greatest {
        args: vec![SqlValueForm::Literal, SqlValueForm::Null]
    }));
    assert!(!form_is_stable(&SqlValueForm::SelfRef {
        column: "b".into()
    }));
    assert!(!form_is_stable(&SqlValueForm::Least {
        args: vec![SqlValueForm::Placeholder, SqlValueForm::Other]
    }));
}

#[test]
fn datetime_idents_and_quoted_now_are_unstable_forms() {
    assert!(matches!(
        super::value::from_expr(&Expr::Identifier(Ident::new("CURRENT_TIMESTAMP"))),
        SqlValueForm::Volatile { name } if name == "current_timestamp"
    ));
    assert_eq!(
        super::value::from_expr(&Expr::Identifier(Ident::new("DEFAULT"))),
        SqlValueForm::Other
    );
    assert_eq!(
        super::value::from_expr(&Expr::Value(
            Value::EscapedStringLiteral("now".into()).with_empty_span()
        )),
        SqlValueForm::Other
    );
    assert_eq!(
        super::value::from_expr(&Expr::Value(
            Value::DollarQuotedString(DollarQuotedString {
                value: "today".into(),
                tag: None,
            })
            .with_empty_span()
        )),
        SqlValueForm::Other
    );
    assert_eq!(
        super::value::from_expr(&Expr::Value(
            Value::UnicodeStringLiteral("now".into()).with_empty_span()
        )),
        SqlValueForm::Other
    );
    let minus_one = Expr::UnaryOp {
        op: UnaryOperator::Minus,
        expr: Box::new(Expr::Value(
            Value::Number("1".into(), false).with_empty_span(),
        )),
    };
    assert_eq!(super::value::from_expr(&minus_one), SqlValueForm::Literal);
    let minus_now = Expr::UnaryOp {
        op: UnaryOperator::Minus,
        expr: Box::new(Expr::Identifier(Ident::new("now"))),
    };
    assert!(matches!(
        super::value::from_expr(&minus_now),
        SqlValueForm::Volatile { name } if name == "now"
    ));
}
