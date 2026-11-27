use super::super::{parse_postgres_sql, parse_postgres_sql_lenient};
use sqlparser::ast::{
    Expr, Function, FunctionArg, FunctionArgOperator, FunctionArguments, JoinOperator, Select,
    SelectItem, SetExpr, Statement, Visit, Visitor,
};
use std::ops::ControlFlow;

pub(super) fn fixture(name: &str) -> &'static str {
    match name {
        "statements" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/rules/postgres-bounded-statements/issue-3397/statements.sql"
        )),
        "invalid" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/rules/postgres-bounded-statements/issue-3397/parser/invalid-left-join.sql"
        )),
        "shape" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/rules/postgres-bounded-statements/issue-3397/parser/assignment-shape.sql"
        )),
        "reopen" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/rules/postgres-bounded-statements/issue-3397-reopen/statements.sql"
        )),
        _ => unreachable!(),
    }
}

pub(super) fn parsed(sql: &str) -> Vec<Statement> {
    parse_postgres_sql(sql).unwrap_or_else(|error| panic!("{error}"))
}

pub(super) fn select_of(statement: &Statement) -> &Select {
    let Statement::Query(query) = statement else {
        panic!("expected a query");
    };
    let SetExpr::Select(select) = query.body.as_ref() else {
        panic!("expected a select");
    };
    select
}

pub(super) fn projected_function(statement: &Statement) -> &Function {
    let expr = match &select_of(statement).projection[0] {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => expr,
        _ => panic!("expected an expression projection"),
    };
    let Expr::Function(function) = expr else {
        panic!("expected a function call, got {expr}");
    };
    function
}

pub(super) fn bare_name(function: &Function) -> String {
    function.name.to_string()
}

pub(super) struct NamedArguments {
    pub(super) arrow_secs: bool,
    pub(super) assignment_secs: bool,
}

impl Visitor for NamedArguments {
    type Break = ();

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<Self::Break> {
        let Expr::Function(function) = expr else {
            return ControlFlow::Continue(());
        };
        if !bare_name(function).eq_ignore_ascii_case("make_interval") {
            return ControlFlow::Continue(());
        }
        let FunctionArguments::List(list) = &function.args else {
            return ControlFlow::Continue(());
        };
        for arg in &list.args {
            match arg {
                FunctionArg::ExprNamed {
                    name,
                    operator: FunctionArgOperator::RightArrow,
                    ..
                } if expr_is_secs(name) => self.arrow_secs = true,
                FunctionArg::ExprNamed {
                    name,
                    operator: FunctionArgOperator::Assignment,
                    ..
                } if expr_is_secs(name) => self.assignment_secs = true,
                _ => {}
            }
        }
        ControlFlow::Continue(())
    }
}

fn expr_is_secs(expr: &Expr) -> bool {
    match expr {
        Expr::Identifier(ident) => ident.value.eq_ignore_ascii_case("secs"),
        other => other.to_string().eq_ignore_ascii_case("secs"),
    }
}

#[test]
fn issue_3397_left_and_right_calls_stay_functions_and_joins_stay_joins() {
    let statements = parsed(fixture("statements"));
    assert_eq!(statements.len(), 9);
    assert!(bare_name(projected_function(&statements[0])).eq_ignore_ascii_case("left"));
    assert!(select_of(&statements[0]).from[0].joins.is_empty());
    assert!(bare_name(projected_function(&statements[1])).eq_ignore_ascii_case("right"));
    assert!(matches!(
        select_of(&statements[6]).from[0].joins[0].join_operator,
        JoinOperator::Left(_) | JoinOperator::LeftOuter(_)
    ));
    assert!(matches!(
        select_of(&statements[7]).from[0].joins[0].join_operator,
        JoinOperator::Right(_) | JoinOperator::RightOuter(_)
    ));
    let error = parse_postgres_sql(fixture("invalid")).expect_err("LEFT ( is a join keyword");
    assert!(
        error.message.contains("JOIN"),
        "bare LEFT before a parenthesis stays a join keyword: {error}"
    );
}

#[test]
fn issue_3397_named_arguments_parse_inside_case() {
    let statements = parsed(fixture("statements"));
    let mut seen = NamedArguments {
        arrow_secs: false,
        assignment_secs: false,
    };
    for statement in &statements {
        let _ = statement.visit(&mut seen);
    }
    assert!(seen.arrow_secs, "secs => value is a named argument");
    assert!(seen.assignment_secs, "secs := value is a named argument");
    let lenient = parse_postgres_sql_lenient("SELECT make_interval(secs := $4);");
    let mut lenient_seen = NamedArguments {
        arrow_secs: false,
        assignment_secs: false,
    };
    for statement in &lenient {
        let _ = statement.visit(&mut lenient_seen);
    }
    assert!(
        lenient_seen.assignment_secs,
        "lenient parse keeps secs := value as a named argument"
    );
}

#[test]
fn issue_3397_current_timestamp_inside_case_is_a_function_call() {
    let statements = parsed(fixture("reopen"));
    let mut calls = 0;
    let mut visitor = CurrentTimestamp { calls: &mut calls };
    for statement in &statements {
        let _ = statement.visit(&mut visitor);
    }
    assert!(calls > 0, "CURRENT_TIMESTAMP parses as a function call");
}

struct CurrentTimestamp<'a> {
    calls: &'a mut usize,
}

impl Visitor for CurrentTimestamp<'_> {
    type Break = ();

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<Self::Break> {
        if let Expr::Function(function) = expr {
            if bare_name(function).eq_ignore_ascii_case("current_timestamp") {
                *self.calls += 1;
            }
        }
        ControlFlow::Continue(())
    }
}
