use super::column_type_name;
use super::{
    extract_create_table_metadata, generated_function, generated_function_arg_columns,
    index_column_name, relation_name,
};
use sqlparser::ast::{
    DataType, Expr, Function, FunctionArgumentClause, FunctionArgumentList, FunctionArguments,
    Ident, IdentWithAlias, IndexColumn, ObjectName, ObjectNamePart, OrderByExpr, OrderByKind,
    OrderByOptions,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/schema")
        .join(name)
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).expect("fixture")
}

#[test]
fn extracts_generated_column_and_quoted_table_name() {
    let tables = extract_create_table_metadata(&read_fixture("generated-column.sql")).unwrap();
    assert_eq!(tables.len(), 1);
    let table = &tables[0];
    assert_eq!(table.table_name, "MixedCase");
    assert_eq!(
        table
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["id", "created_at", "note"]
    );
    let created = &table.columns[1];
    assert!(created.is_generated);
    assert_eq!(created.constraints, ["CONSTR_GENERATED"]);
    assert_eq!(
        created.generated_function.as_deref(),
        Some("uuid_extract_timestamp")
    );
    assert_eq!(created.generated_function_arg_columns, ["id"]);
    assert!(created
        .generated_expression
        .as_deref()
        .is_some_and(|expr| expr.contains("uuid_extract_timestamp")));
    assert!(table.columns[0].is_primary_key);
    assert!(!created.is_primary_key);
}

#[test]
fn recognizes_table_level_primary_key() {
    let tables = extract_create_table_metadata(&read_fixture("table-level-pk.sql")).unwrap();
    let id = tables[0].columns.iter().find(|c| c.name == "id").unwrap();
    let name = tables[0].columns.iter().find(|c| c.name == "name").unwrap();
    assert!(id.is_primary_key);
    assert!(!name.is_primary_key);
    assert!(id.constraints.contains(&"CONSTR_NOTNULL".to_string()));
    assert!(id.constraints.contains(&"CONSTR_DEFAULT".to_string()));
}

#[test]
fn extracts_additional_column_constraints() {
    let tables = extract_create_table_metadata(&read_fixture("constraints.sql")).unwrap();
    let table = &tables[0];
    assert_eq!(table.table_name, "constraint_kitchen");
    let by_name = |name: &str| table.columns.iter().find(|c| c.name == name).unwrap();
    assert!(by_name("email")
        .constraints
        .contains(&"CONSTR_UNIQUE".to_string()));
    assert!(by_name("nickname")
        .constraints
        .contains(&"CONSTR_NULL".to_string()));
    assert!(by_name("parent_id")
        .constraints
        .contains(&"CONSTR_FOREIGN".to_string()));
    assert!(by_name("score")
        .constraints
        .contains(&"CONSTR_CHECK".to_string()));
    assert!(
        by_name("serial_id").is_generated
            || by_name("serial_id")
                .constraints
                .contains(&"CONSTR_IDENTITY".to_string())
    );
    assert_eq!(
        by_name("computed").generated_function.as_deref(),
        Some("now")
    );
    assert!(by_name("computed")
        .generated_function_arg_columns
        .is_empty());
    assert_eq!(
        by_name("nested_gen").generated_function.as_deref(),
        Some("uuid_extract_timestamp")
    );
    assert!(!by_name("skipped").is_generated);
    assert!(by_name("id").type_name.is_some());
}

#[test]
fn unparseable_sql_returns_error() {
    let error = extract_create_table_metadata(&read_fixture("invalid.sql")).expect_err("invalid");
    assert!(!error.message.is_empty());
}

#[test]
fn non_create_statements_are_ignored() {
    let tables = extract_create_table_metadata("SELECT 1; COMMENT ON TABLE t IS 'x';").unwrap();
    assert!(tables.is_empty());
}

#[test]
fn generated_helpers_cover_non_function_shapes() {
    let ident = Expr::Identifier(Ident::new("id"));
    assert!(generated_function(&ident).is_none());
    assert!(generated_function_arg_columns(&ident).is_empty());
    let subquery = Expr::Function(Function {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new("now"))]),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::Subquery(Box::new(sqlparser::ast::Query {
            with: None,
            body: Box::new(sqlparser::ast::SetExpr::Values(sqlparser::ast::Values {
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
        })),
        filter: None,
        null_treatment: None,
        over: None,
        within_group: Vec::new(),
    });
    assert_eq!(generated_function(&subquery).as_deref(), Some("now"));
    assert!(generated_function_arg_columns(&subquery).is_empty());
}

#[test]
fn index_column_name_reads_identifiers() {
    let simple = IndexColumn {
        column: OrderByExpr {
            expr: Expr::Identifier(Ident::new("id")),
            options: OrderByOptions {
                asc: None,
                nulls_first: None,
            },
            with_fill: None,
        },
        operator_class: None,
    };
    assert_eq!(index_column_name(&simple).as_deref(), Some("id"));
    let compound = IndexColumn {
        column: OrderByExpr {
            expr: Expr::CompoundIdentifier(vec![Ident::new("public"), Ident::new("id")]),
            options: OrderByOptions {
                asc: None,
                nulls_first: None,
            },
            with_fill: None,
        },
        operator_class: None,
    };
    assert_eq!(index_column_name(&compound).as_deref(), Some("id"));
    let other = IndexColumn {
        column: OrderByExpr {
            expr: Expr::Value(sqlparser::ast::Value::Number("1".into(), false).into()),
            options: OrderByOptions {
                asc: None,
                nulls_first: None,
            },
            with_fill: None,
        },
        operator_class: None,
    };
    assert!(index_column_name(&other).is_none());
}

#[test]
fn relation_name_uses_last_identifier() {
    let name = ObjectName(vec![
        ObjectNamePart::Identifier(Ident::new("public")),
        ObjectNamePart::Identifier(Ident::new("users")),
    ]);
    assert_eq!(relation_name(&name), "users");
    assert_eq!(relation_name(&ObjectName(Vec::new())), "");
}

#[test]
fn function_argument_column_names_skip_wildcards() {
    let expr = Expr::Function(Function {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new("fn"))]),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::List(FunctionArgumentList {
            duplicate_treatment: None,
            args: vec![
                sqlparser::ast::FunctionArg::Unnamed(sqlparser::ast::FunctionArgExpr::Wildcard),
                sqlparser::ast::FunctionArg::Named {
                    name: Ident::new("col"),
                    arg: sqlparser::ast::FunctionArgExpr::Expr(Expr::Identifier(Ident::new("Id"))),
                    operator: sqlparser::ast::FunctionArgOperator::Equals,
                },
            ],
            clauses: vec![FunctionArgumentClause::IgnoreOrRespectNulls(
                sqlparser::ast::NullTreatment::IgnoreNulls,
            )],
        }),
        filter: None,
        null_treatment: None,
        over: None,
        within_group: Vec::new(),
    });
    assert_eq!(generated_function_arg_columns(&expr), ["id"]);
    let named_expr = Expr::Function(Function {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new("fn"))]),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::List(FunctionArgumentList {
            duplicate_treatment: None,
            args: vec![sqlparser::ast::FunctionArg::ExprNamed {
                name: Expr::Identifier(Ident::new("alias")),
                arg: sqlparser::ast::FunctionArgExpr::Expr(Expr::Identifier(Ident::new("Col"))),
                operator: sqlparser::ast::FunctionArgOperator::Equals,
            }],
            clauses: Vec::new(),
        }),
        filter: None,
        null_treatment: None,
        over: None,
        within_group: Vec::new(),
    });
    assert_eq!(generated_function_arg_columns(&named_expr), ["col"]);
    let empty_name = Expr::Function(Function {
        name: ObjectName(Vec::new()),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::None,
        filter: None,
        null_treatment: None,
        over: None,
        within_group: Vec::new(),
    });
    assert!(generated_function(&empty_name).is_none());
    let nested = Expr::Nested(Box::new(Expr::Function(Function {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new("now"))]),
        uses_odbc_syntax: false,
        parameters: FunctionArguments::None,
        args: FunctionArguments::None,
        filter: None,
        null_treatment: None,
        over: None,
        within_group: Vec::new(),
    })));
    assert_eq!(generated_function(&nested).as_deref(), Some("now"));
    let _ = IdentWithAlias {
        ident: Ident::new("x"),
        alias: Ident::new("y"),
    };
    let _ = OrderByKind::All(OrderByOptions {
        asc: None,
        nulls_first: None,
    });
}

#[test]
fn unspecified_type_is_none() {
    assert!(column_type_name(&DataType::Unspecified).is_none());
    assert!(column_type_name(&DataType::Uuid).is_some());
}

#[test]
fn ignored_options_and_table_constraints_do_not_panic() {
    let tables = extract_create_table_metadata(
        "CREATE TABLE t (id text COLLATE \"C\", name text, UNIQUE (name));",
    )
    .unwrap();
    assert_eq!(tables[0].table_name, "t");
    assert!(!tables[0].columns[0].is_primary_key);
}

#[test]
fn relation_name_skips_non_identifier_parts() {
    let name = ObjectName(vec![ObjectNamePart::Function(
        sqlparser::ast::ObjectNamePartFunction {
            name: Ident::new("fn"),
            args: Vec::new(),
        },
    )]);
    assert_eq!(relation_name(&name), "");
}
