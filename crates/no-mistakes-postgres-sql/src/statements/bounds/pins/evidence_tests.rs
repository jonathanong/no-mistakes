use super::{is_literal, placeholder_binding, sole_argument, strict_name};
use crate::parse_postgres_expression;
use sqlparser::ast::{
    Expr, Function, FunctionArg, FunctionArgExpr, FunctionArgOperator, FunctionArguments, Ident,
};

fn function(sql: &str) -> Function {
    let Some(Expr::Function(function)) = parse_postgres_expression(sql) else {
        panic!("expected a function expression");
    };
    function
}

#[test]
fn strict_names_are_unquoted_lower_upper_or_btrim() {
    assert_eq!(
        strict_name(&function("lower(username)")).as_deref(),
        Some("lower")
    );
    assert_eq!(
        strict_name(&function("pg_catalog.upper(username)")).as_deref(),
        Some("upper")
    );
    assert_eq!(
        strict_name(&function("\"lower\"(username)")).as_deref(),
        None
    );
    assert_eq!(
        strict_name(&function("app.lower(username)")).as_deref(),
        None
    );
    assert_eq!(
        strict_name(&function("lower(username) FILTER (WHERE true)")).as_deref(),
        None
    );
    assert_eq!(
        strict_name(&function("lower(username) OVER ()")).as_deref(),
        None
    );
    assert!(sole_argument(&function("btrim(username)")).is_some());
    assert!(sole_argument(&function("btrim(username, 'x')")).is_none());
    let mut bare = function("lower(username)");
    bare.args = FunctionArguments::None;
    assert!(sole_argument(&bare).is_none());
    let mut named = function("lower(username)");
    if let FunctionArguments::List(arguments) = &mut named.args {
        arguments.args = vec![FunctionArg::Named {
            name: Ident::new("username"),
            arg: FunctionArgExpr::Expr(Expr::Identifier(Ident::new("username"))),
            operator: FunctionArgOperator::Equals,
        }];
    }
    assert!(sole_argument(&named).is_none());
}

fn parsed(sql: &str) -> Expr {
    parse_postgres_expression(sql).unwrap_or_else(|| panic!("parse {sql}"))
}

#[test]
fn placeholder_bindings_require_one_placeholder_and_one_literal() {
    assert!(placeholder_binding(&parsed("$1 = 'pending'")).is_some());
    assert!(placeholder_binding(&parsed("'pending' = $2")).is_some());
    assert!(placeholder_binding(&parsed("$1 = 'pending'::text")).is_some());
    assert!(placeholder_binding(&parsed("true = $1")).is_some());
    assert!(placeholder_binding(&parsed("$1 = 1")).is_some());
    assert!(placeholder_binding(&parsed("$1 = $2")).is_none());
    assert!(placeholder_binding(&parsed("status = 'pending'")).is_none());
    assert!(placeholder_binding(&parsed("$1 = lower('pending')")).is_none());
    assert!(is_literal(&parsed("'pending'")));
    assert!(is_literal(&parsed("true")));
    assert!(is_literal(&parsed("1")));
    assert!(is_literal(&parsed("'pending'::varchar(3)")));
    assert!(!is_literal(&parsed("$1")));
    assert!(!is_literal(&parsed("status")));
    assert!(!is_literal(&parsed("lower('pending')")));
}
